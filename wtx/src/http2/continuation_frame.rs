use crate::{
  http::{Header, Headers, KnownHeaderName},
  http2::{FrameInit, FrameInitTy, HpackDecoder, HpackHeaderBasic, Http2Error, EOH_MASK, U31},
  misc::Usize,
};

#[derive(Debug)]
pub(crate) struct ContinuationFrame {
  flag: u8,
  is_over_size: bool,
  stream_id: U31,
}

impl ContinuationFrame {
  pub(crate) fn new(stream_id: U31) -> Self {
    Self { flag: 0, is_over_size: false, stream_id }
  }

  pub(crate) fn bytes(&self) -> [u8; 9] {
    FrameInit::new(0, self.flag, self.stream_id, FrameInitTy::Continuation).bytes()
  }

  pub(crate) fn is_eoh(&self) -> bool {
    self.flag & EOH_MASK == EOH_MASK
  }

  #[inline]
  pub(crate) fn is_over_size(&self) -> bool {
    self.is_over_size
  }

  pub(crate) fn read<const IS_TRAILER: bool>(
    data: &[u8],
    fi: FrameInit,
    headers: &mut Headers,
    headers_size: &mut usize,
    hpack_dec: &mut HpackDecoder,
  ) -> crate::Result<Self> {
    if fi.stream_id.is_zero() {
      return Err(crate::Error::http2_go_away_generic(Http2Error::InvalidContinuationFrameZeroId));
    }
    let mut is_malformed = false;
    let mut is_over_size = false;
    let max_header_list_size = *Usize::from(hpack_dec.max_bytes());
    hpack_dec.decode(data, |(elem, name, value)| {
      match elem {
        HpackHeaderBasic::Field => match KnownHeaderName::try_from(name) {
          Ok(
            KnownHeaderName::Connection
            | KnownHeaderName::KeepAlive
            | KnownHeaderName::ProxyConnection
            | KnownHeaderName::TransferEncoding
            | KnownHeaderName::Upgrade,
          ) => {
            is_malformed = true;
          }
          Ok(KnownHeaderName::Te) if value != b"trailers" => {
            is_malformed = true;
          }
          _ => {
            let len = decoded_header_size(name.len(), value.len());
            *headers_size = headers_size.wrapping_add(len);
            is_over_size = *headers_size >= max_header_list_size;
            if !is_over_size {
              headers.reserve(name.len().wrapping_add(value.len()), 1);
              headers.push_front(Header {
                is_sensitive: false,
                is_trailer: IS_TRAILER,
                name,
                value,
              })?;
            }
          }
        },
        _ => {
          is_malformed = true;
        }
      }
      Ok(())
    })?;
    Ok(Self { flag: fi.flags, is_over_size, stream_id: fi.stream_id })
  }

  pub(crate) fn set_eoh(&mut self) {
    self.flag |= EOH_MASK;
  }
}

#[inline]
fn decoded_header_size(name: usize, value: usize) -> usize {
  name.wrapping_add(value).wrapping_add(32)
}
