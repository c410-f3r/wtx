use crate::{
  misc::net::{PartitionedFilledBuffer, read_header},
  stream::StreamReader,
  web_socket::{
    FIN_MASK, MAX_CONTROL_PAYLOAD_LEN, OpCode, PAYLOAD_MASK, RSV1_MASK, RSV2_MASK, RSV3_MASK,
    WebSocketError,
    compression::NegotiatedCompression,
    misc::{has_masked_frame, op_code},
  },
};

/// Parameters of an WebSocket frame.
#[derive(Debug)]
pub(crate) struct ReadFrameInfo {
  pub(crate) fin: bool,
  pub(crate) header_len: u8,
  pub(crate) mask: Option<[u8; 4]>,
  pub(crate) op_code: OpCode,
  pub(crate) payload_len: usize,
  pub(crate) should_decompress: bool,
}

impl ReadFrameInfo {
  /// Creates a new instance based on a sequence of bytes.
  #[cfg(feature = "http2")]
  #[inline]
  pub(crate) fn from_bytes<NC, const IS_CLIENT: bool>(
    bytes: &mut &[u8],
    max_payload_len: usize,
    nc_rsv1: u8,
    no_masking: bool,
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
    let tuple = Self::manage_first_two_bytes::<NC>(first_two, nc_rsv1)?;
    let (fin, length_code, masked, op_code, should_decompress) = tuple;
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
    let mask = if Self::manage_mask::<IS_CLIENT>(masked, no_masking)? {
      let [a, b, c, d, rest @ ..] = bytes else {
        return Err(crate::Error::UnexpectedBufferState);
      };
      *bytes = rest;
      header_len = header_len.wrapping_add(4);
      Some([*a, *b, *c, *d])
    } else {
      None
    };
    Self::manage_final_params(fin, op_code, max_payload_len, payload_len)?;
    Ok(ReadFrameInfo { fin, header_len, mask, op_code, payload_len, should_decompress })
  }

  pub(crate) async fn from_stream<NC, SR, const IS_CLIENT: bool>(
    max_payload_len: usize,
    nc_rsv1: u8,
    network_buffer: &mut PartitionedFilledBuffer,
    no_masking: bool,
    read: &mut usize,
    stream: &mut SR,
  ) -> crate::Result<Self>
  where
    NC: NegotiatedCompression,
    SR: StreamReader,
  {
    let buffer = network_buffer.following_rest_mut();
    let first_two = read_header::<0, 2, SR>(buffer, read, stream).await?;
    let tuple = Self::manage_first_two_bytes::<NC>(first_two, nc_rsv1)?;
    let (fin, length_code, masked, op_code, should_decompress) = tuple;
    let mut mask = None;
    let (header_len, payload_len) = match length_code {
      126 => {
        let payload_len = read_header::<2, 2, SR>(buffer, read, stream).await?;
        if Self::manage_mask::<IS_CLIENT>(masked, no_masking)? {
          mask = Some(read_header::<4, 4, SR>(buffer, read, stream).await?);
          (8, u16::from_be_bytes(payload_len).into())
        } else {
          (4, u16::from_be_bytes(payload_len).into())
        }
      }
      127 => {
        let payload_len = read_header::<2, 8, SR>(buffer, read, stream).await?;
        if Self::manage_mask::<IS_CLIENT>(masked, no_masking)? {
          mask = Some(read_header::<10, 4, SR>(buffer, read, stream).await?);
          (14, u64::from_be_bytes(payload_len).try_into()?)
        } else {
          (10, u64::from_be_bytes(payload_len).try_into()?)
        }
      }
      _ => {
        if Self::manage_mask::<IS_CLIENT>(masked, no_masking)? {
          mask = Some(read_header::<2, 4, SR>(buffer, read, stream).await?);
          (6, length_code.into())
        } else {
          (2, length_code.into())
        }
      }
    };
    Self::manage_final_params(fin, op_code, max_payload_len, payload_len)?;
    Ok(ReadFrameInfo { fin, header_len, mask, op_code, payload_len, should_decompress })
  }

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

  fn manage_first_two_bytes<NC>(
    [a, b]: [u8; 2],
    nc_rsv1: u8,
  ) -> crate::Result<(bool, u8, bool, OpCode, bool)>
  where
    NC: NegotiatedCompression,
  {
    let rsv1 = a & RSV1_MASK;
    let rsv2 = a & RSV2_MASK;
    let rsv3 = a & RSV3_MASK;
    if rsv2 != 0 || rsv3 != 0 {
      return Err(WebSocketError::InvalidCompressionHeaderParameter.into());
    }
    let should_decompress = if NC::IS_NOOP {
      false
    } else if nc_rsv1 == 0 {
      if rsv1 != 0 {
        return Err(WebSocketError::InvalidCompressionHeaderParameter.into());
      }
      false
    } else {
      rsv1 != 0
    };
    let fin = a & FIN_MASK != 0;
    let length_code = b & PAYLOAD_MASK;
    let masked = has_masked_frame(b);
    let op_code = op_code(a)?;
    Ok((fin, length_code, masked, op_code, should_decompress))
  }

  fn manage_mask<const IS_CLIENT: bool>(masked: bool, no_masking: bool) -> crate::Result<bool> {
    Ok(if IS_CLIENT {
      false
    } else if no_masking {
      if masked {
        return Err(WebSocketError::InvalidMaskBit.into());
      }
      false
    } else {
      if !masked {
        return Err(WebSocketError::InvalidMaskBit.into());
      }
      true
    })
  }
}
