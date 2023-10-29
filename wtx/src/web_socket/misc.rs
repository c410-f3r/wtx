use crate::web_socket::{FrameBuffer, OpCode, WebSocketError, MAX_HDR_LEN_U8};

pub(crate) fn define_fb_from_header_params<B, const IS_CLIENT: bool>(
  fb: &mut FrameBuffer<B>,
  fin: bool,
  header_buffer_len: Option<u8>,
  op_code: OpCode,
  payload_len: usize,
  rsv1: u8,
) -> crate::Result<()>
where
  B: AsMut<[u8]> + AsRef<[u8]>,
{
  let new_header_len = header_len_from_payload_len::<IS_CLIENT>(payload_len);
  let (buffer, header_begin_idx) = if let Some(el) = header_buffer_len {
    let header_begin_idx = el.saturating_sub(new_header_len);
    let buffer = fb.buffer_mut().as_mut().get_mut(header_begin_idx.into()..).unwrap_or_default();
    (buffer, header_begin_idx)
  } else {
    (fb.buffer_mut().as_mut(), 0)
  };
  let _ = copy_header_params_to_buffer::<IS_CLIENT>(buffer, fin, op_code, payload_len, rsv1)?;
  fb.set_indices(header_begin_idx, new_header_len, payload_len)?;
  Ok(())
}

pub(crate) const fn header_placeholder<const IS_CLIENT: bool>() -> u8 {
  if IS_CLIENT {
    MAX_HDR_LEN_U8
  } else {
    MAX_HDR_LEN_U8 - 4
  }
}

pub(crate) fn op_code(first_header_byte: u8) -> crate::Result<OpCode> {
  OpCode::try_from(first_header_byte & 0b0000_1111)
}

fn copy_header_params_to_buffer<const IS_CLIENT: bool>(
  buffer: &mut [u8],
  fin: bool,
  op_code: OpCode,
  payload_len: usize,
  rsv1: u8,
) -> crate::Result<u8> {
  fn first_header_byte(fin: bool, op_code: OpCode, rsv1: u8) -> u8 {
    u8::from(fin) << 7 | rsv1 | u8::from(op_code)
  }

  fn manage_mask<const IS_CLIENT: bool, const N: u8>(
    rest: &mut [u8],
    second_byte: &mut u8,
  ) -> crate::Result<u8> {
    Ok(if IS_CLIENT {
      *second_byte &= 0b0111_1111;
      let [a, b, c, d, ..] = rest else {
        return Err(WebSocketError::InvalidFrameHeaderBounds.into());
      };
      *a = 0;
      *b = 0;
      *c = 0;
      *d = 0;
      N.wrapping_add(4)
    } else {
      N
    })
  }

  match payload_len {
    0..=125 => {
      if let ([a, b, rest @ ..], Ok(u8_len)) = (buffer, u8::try_from(payload_len)) {
        *a = first_header_byte(fin, op_code, rsv1);
        *b = u8_len;
        return manage_mask::<IS_CLIENT, 2>(rest, b);
      }
    }
    126..=0xFFFF => {
      let rslt = u16::try_from(payload_len).map(u16::to_be_bytes);
      if let ([a, b, c, d, rest @ ..], Ok([len_c, len_d])) = (buffer, rslt) {
        *a = first_header_byte(fin, op_code, rsv1);
        *b = 126;
        *c = len_c;
        *d = len_d;
        return manage_mask::<IS_CLIENT, 4>(rest, b);
      }
    }
    _ => {
      if let (
        [a, b, c, d, e, f, g, h, i, j, rest @ ..],
        Ok([len_c, len_d, len_e, len_f, len_g, len_h, len_i, len_j]),
      ) = (buffer, u64::try_from(payload_len).map(u64::to_be_bytes))
      {
        *a = first_header_byte(fin, op_code, rsv1);
        *b = 127;
        *c = len_c;
        *d = len_d;
        *e = len_e;
        *f = len_f;
        *g = len_g;
        *h = len_h;
        *i = len_i;
        *j = len_j;
        return manage_mask::<IS_CLIENT, 10>(rest, b);
      }
    }
  }

  Err(WebSocketError::InvalidFrameHeaderBounds.into())
}

fn header_len_from_payload_len<const IS_CLIENT: bool>(payload_len: usize) -> u8 {
  let mask_len = if IS_CLIENT { 4 } else { 0 };
  let n: u8 = match payload_len {
    0..=125 => 2,
    126..=0xFFFF => 4,
    _ => 10,
  };
  n.wrapping_add(mask_len)
}
