use crate::{
  misc::{PartitionedFilledBuffer, Stream, _read_until},
  web_socket::{
    compression::NegotiatedCompression, misc::op_code, OpCode, WebSocketError,
    MAX_CONTROL_PAYLOAD_LEN,
  },
};

/// Parameters of an WebSocket frame.
#[derive(Debug)]
pub struct ReadFrameInfo {
  pub(crate) fin: bool,
  pub(crate) header_len: u8,
  pub(crate) mask: Option<[u8; 4]>,
  pub(crate) op_code: OpCode,
  pub(crate) payload_len: usize,
  pub(crate) should_decompress: bool,
}

impl ReadFrameInfo {
  /// Creates a new instance based on a sequence of bytes.
  #[inline]
  pub fn from_bytes<NC, const IS_CLIENT: bool>(
    bytes: &mut &[u8],
    max_payload_len: usize,
    nc: &NC,
  ) -> crate::Result<Self>
  where
    NC: NegotiatedCompression,
  {
    let first_two = {
      let [a, b, rest @ ..] = bytes else {
        return Err(crate::Error::UnexpectedBufferState);
      };
      *bytes = rest;
      [*a, *b]
    };
    let tuple = Self::manage_first_two_bytes(first_two, nc)?;
    let (fin, length_code, op_code, should_decompress) = tuple;
    let (mut header_len, payload_len) = match length_code {
      126 => {
        let [a, b, rest @ ..] = bytes else {
          return Err(crate::Error::UnexpectedBufferState);
        };
        *bytes = rest;
        (4u8, u16::from_be_bytes([*a, *b]).into())
      }
      127 => {
        let [a, b, c, d, e, f, g, h, rest @ ..] = bytes else {
          return Err(crate::Error::UnexpectedBufferState);
        };
        *bytes = rest;
        (10, u64::from_be_bytes([*a, *b, *c, *d, *e, *f, *g, *h]).try_into()?)
      }
      _ => (2, length_code.into()),
    };
    let mut mask = None;
    if !IS_CLIENT {
      let [a, b, c, d, rest @ ..] = bytes else {
        return Err(crate::Error::UnexpectedBufferState);
      };
      *bytes = rest;
      mask = Some([*a, *b, *c, *d]);
      header_len = header_len.wrapping_add(4);
    }
    Self::manage_final_params(fin, op_code, max_payload_len, payload_len)?;
    Ok(ReadFrameInfo { fin, header_len, mask, op_code, payload_len, should_decompress })
  }

  #[inline]
  pub(crate) async fn from_stream<NC, S, const IS_CLIENT: bool>(
    max_payload_len: usize,
    nc: &NC,
    network_buffer: &mut PartitionedFilledBuffer,
    read: &mut usize,
    stream: &mut S,
  ) -> crate::Result<Self>
  where
    NC: NegotiatedCompression,
    S: Stream,
  {
    let buffer = network_buffer._following_rest_mut();
    let first_two = _read_until::<2, S>(buffer, read, 0, stream).await?;
    let tuple = Self::manage_first_two_bytes(first_two, nc)?;
    let (fin, length_code, op_code, should_decompress) = tuple;
    let (mut header_len, payload_len) = match length_code {
      126 => {
        let payload_len = _read_until::<2, S>(buffer, read, 2, stream).await?;
        (4u8, u16::from_be_bytes(payload_len).into())
      }
      127 => {
        let payload_len = _read_until::<8, S>(buffer, read, 2, stream).await?;
        (10, u64::from_be_bytes(payload_len).try_into()?)
      }
      _ => (2, length_code.into()),
    };
    let mut mask = None;
    if !IS_CLIENT {
      mask = Some(_read_until::<4, S>(buffer, read, header_len.into(), stream).await?);
      header_len = header_len.wrapping_add(4);
    }
    Self::manage_final_params(fin, op_code, max_payload_len, payload_len)?;
    Ok(ReadFrameInfo { fin, header_len, mask, op_code, payload_len, should_decompress })
  }

  #[inline]
  fn manage_final_params(
    fin: bool,
    op_code: OpCode,
    max_payload_len: usize,
    payload_len: usize,
  ) -> crate::Result<()> {
    if op_code.is_control() && !fin {
      return Err(WebSocketError::UnexpectedFragmentedControlFrame.into());
    }
    if op_code == OpCode::Ping && payload_len > MAX_CONTROL_PAYLOAD_LEN {
      return Err(WebSocketError::VeryLargeControlFrame.into());
    }
    if payload_len >= max_payload_len {
      return Err(WebSocketError::VeryLargePayload.into());
    }
    Ok(())
  }

  #[inline]
  fn manage_first_two_bytes<NC>([a, b]: [u8; 2], nc: &NC) -> crate::Result<(bool, u8, OpCode, bool)>
  where
    NC: NegotiatedCompression,
  {
    let rsv1 = a & 0b0100_0000;
    let rsv2 = a & 0b0010_0000;
    let rsv3 = a & 0b0001_0000;
    if rsv2 != 0 || rsv3 != 0 {
      return Err(WebSocketError::InvalidCompressionHeaderParameter.into());
    }
    let should_decompress = if NC::IS_NOOP {
      false
    } else if nc.rsv1() == 0 {
      if rsv1 != 0 {
        return Err(WebSocketError::InvalidCompressionHeaderParameter.into());
      }
      false
    } else {
      rsv1 != 0
    };
    let fin = a & 0b1000_0000 != 0;
    let length_code = b & 0b0111_1111;
    let op_code = op_code(a)?;
    Ok((fin, length_code, op_code, should_decompress))
  }
}
