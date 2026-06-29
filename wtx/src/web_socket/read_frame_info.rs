use crate::{
  stream::{BufStreamReader, StreamReader},
  web_socket::{
    FIN_MASK, MAX_CONTROL_PAYLOAD_LEN, OpCode, PAYLOAD_MASK, RSV1_MASK, RSV2_MASK, RSV3_MASK,
    WebSocketError,
    misc::{has_masked_frame, op_code},
    web_socket_compression::WebSocketDecompression,
  },
};

/// Parameters of an WebSocket frame.
#[derive(Debug)]
pub(crate) struct ReadFrameInfo {
  pub(crate) fin: bool,
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
    NC: crate::web_socket::web_socket_compression::NegotiatedWsCompression,
  {
    let first_two = {
      let [b0, b1, rest @ ..] = bytes else {
        return Err(crate::Error::UnexpectedBufferState);
      };
      *bytes = rest;
      [*b0, *b1]
    };
    let tuple = Self::manage_first_two_bytes::<NC>(first_two, nc_rsv1)?;
    let (fin, length_code, masked, op_code, should_decompress) = tuple;
    let payload_len = match length_code {
      126 => {
        let [b0, b1, rest @ ..] = bytes else {
          return Err(crate::Error::UnexpectedBufferState);
        };
        *bytes = rest;
        u16::from_be_bytes([*b0, *b1]).into()
      }
      127 => {
        let [b0, b1, b2, b3, b4, b5, b6, b7, rest @ ..] = bytes else {
          return Err(crate::Error::UnexpectedBufferState);
        };
        *bytes = rest;
        u64::from_be_bytes([*b0, *b1, *b2, *b3, *b4, *b5, *b6, *b7]).try_into()?
      }
      _ => length_code.into(),
    };
    let mask = if Self::manage_mask::<IS_CLIENT>(masked, no_masking)? {
      let [b0, b1, b2, b3, rest @ ..] = bytes else {
        return Err(crate::Error::UnexpectedBufferState);
      };
      *bytes = rest;
      Some([*b0, *b1, *b2, *b3])
    } else {
      None
    };
    Self::manage_final_params(fin, op_code, max_payload_len, payload_len)?;
    Ok(ReadFrameInfo { fin, mask, op_code, payload_len, should_decompress })
  }

  pub(crate) async fn from_stream<D, SR, const IS_CLIENT: bool>(
    max_payload_len: usize,
    nc_rsv1: u8,
    network_buffer: &mut BufStreamReader,
    no_masking: bool,
    stream: &mut SR,
  ) -> crate::Result<Self>
  where
    D: WebSocketDecompression,
    SR: StreamReader,
  {
    let first_two = network_buffer.read_header::<_, 2>(stream).await?.rslt()?;
    let tuple = Self::manage_first_two_bytes::<D>(first_two, nc_rsv1)?;
    let (fin, length_code, masked, op_code, should_decompress) = tuple;
    let mut mask = None;
    let payload_len = match length_code {
      126 => {
        let payload_len = network_buffer.read_header::<_, 2>(stream).await?.rslt()?;
        if Self::manage_mask::<IS_CLIENT>(masked, no_masking)? {
          mask = Some(network_buffer.read_header::<_, 4>(stream).await?.rslt()?);
          u16::from_be_bytes(payload_len).into()
        } else {
          u16::from_be_bytes(payload_len).into()
        }
      }
      127 => {
        let payload_len = network_buffer.read_header::<_, 8>(stream).await?.rslt()?;
        if Self::manage_mask::<IS_CLIENT>(masked, no_masking)? {
          mask = Some(network_buffer.read_header::<_, 4>(stream).await?.rslt()?);
          u64::from_be_bytes(payload_len).try_into()?
        } else {
          u64::from_be_bytes(payload_len).try_into()?
        }
      }
      _ => {
        if Self::manage_mask::<IS_CLIENT>(masked, no_masking)? {
          mask = Some(network_buffer.read_header::<_, 4>(stream).await?.rslt()?);
          length_code.into()
        } else {
          length_code.into()
        }
      }
    };
    Self::manage_final_params(fin, op_code, max_payload_len, payload_len)?;
    Ok(ReadFrameInfo { fin, mask, op_code, payload_len, should_decompress })
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

  fn manage_first_two_bytes<D>(
    [b0, b1]: [u8; 2],
    nc_rsv1: u8,
  ) -> crate::Result<(bool, u8, bool, OpCode, bool)>
  where
    D: WebSocketDecompression,
  {
    let rsv1 = b0 & RSV1_MASK;
    let rsv2 = b0 & RSV2_MASK;
    let rsv3 = b0 & RSV3_MASK;
    if rsv2 != 0 || rsv3 != 0 {
      return Err(WebSocketError::InvalidCompressionHeaderParameter.into());
    }
    let should_decompress = if D::IS_NOOP {
      false
    } else if nc_rsv1 == 0 {
      if rsv1 != 0 {
        return Err(WebSocketError::InvalidCompressionHeaderParameter.into());
      }
      false
    } else {
      rsv1 != 0
    };
    let fin = b0 & FIN_MASK != 0;
    let length_code = b1 & PAYLOAD_MASK;
    let masked = has_masked_frame(b1);
    let op_code = op_code(b0)?;
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
