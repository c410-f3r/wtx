use crate::{
  http::{HeaderName, Headers},
  http2::{FrameHeaderTy, FrameInit, HpackDecoder, HpackHeaderBasic, EOH_MASK, U31},
  misc::Usize,
};

#[derive(Debug)]
pub(crate) struct ContinuationFrame {
  flag: u8,
  stream_id: U31,
}

impl ContinuationFrame {
  pub(crate) fn new(stream_id: U31) -> Self {
    Self { flag: 0, stream_id }
  }

  pub(crate) fn bytes(&self) -> [u8; 9] {
    FrameInit::new(0, self.flag, self.stream_id, FrameHeaderTy::Continuation).bytes()
  }

  pub(crate) fn is_eoh(&self) -> bool {
    self.flag & EOH_MASK == EOH_MASK
  }

  pub(crate) fn read(
    data: &[u8],
    fi: FrameInit,
    headers: &mut Headers,
    headers_size: &mut usize,
    hpack_dec: &mut HpackDecoder,
  ) -> crate::Result<Self> {
    let mut is_malformed = false;
    if fi.stream_id.is_zero() {
      return Err(crate::http2::ErrorCode::FrameSizeError.into());
    }
    let max_header_list_size = *Usize::from(hpack_dec.max_bytes());
    hpack_dec.decode(data, |(elem, name, value)| {
      match elem {
        HpackHeaderBasic::Field => match HeaderName::new(name) {
          HeaderName::CONNECTION
          | HeaderName::KEEP_ALIVE
          | HeaderName::PROXY_CONNECTION
          | HeaderName::TRANSFER_ENCODING
          | HeaderName::UPGRADE => {
            is_malformed = true;
          }
          HeaderName::TE if value != b"trailers" => {
            is_malformed = true;
          }
          _ => {
            let len = decoded_header_size(name.len(), value.len());
            *headers_size = headers_size.wrapping_add(len);
            let is_over_size = *headers_size >= max_header_list_size;
            if !is_over_size {
              headers.push_front(name, value, false)?;
            }
          }
        },
        _ => {
          is_malformed = true;
        }
      }
      Ok(())
    })?;

    Ok(Self { flag: fi.flags, stream_id: fi.stream_id })
  }

  pub(crate) fn set_eoh(&mut self) {
    self.flag |= EOH_MASK;
  }
}

#[inline]
fn decoded_header_size(name: usize, value: usize) -> usize {
  name.wrapping_add(value).wrapping_add(32)
}
