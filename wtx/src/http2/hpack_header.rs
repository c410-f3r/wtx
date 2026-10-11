use crate::{
  http::{Method, Protocol, StatusCode},
  http2::{Http2Error, misc::protocol_err},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HpackHeaderBasic {
  Authority,
  Field,
  Method(Method),
  Path,
  Protocol(Protocol),
  Scheme,
  StatusCode(StatusCode),
}

impl HpackHeaderBasic {
  pub(crate) const fn len(self, name: &str, value: &str) -> usize {
    match self {
      HpackHeaderBasic::Authority => 10usize.wrapping_add(value.len()).wrapping_add(32),
      HpackHeaderBasic::Field => name.len().wrapping_add(value.len()).wrapping_add(32),
      HpackHeaderBasic::Method(_) => 6usize.wrapping_add(value.len()).wrapping_add(32),
      HpackHeaderBasic::Path => 5usize.wrapping_add(value.len()).wrapping_add(32),
      HpackHeaderBasic::Protocol(_) => 9usize.wrapping_add(value.len()).wrapping_add(32),
      HpackHeaderBasic::Scheme => 7usize.wrapping_add(value.len()).wrapping_add(32),
      HpackHeaderBasic::StatusCode(_) => 7usize.wrapping_add(3).wrapping_add(32),
    }
  }
}

impl TryFrom<(HpackHeaderName, &str)> for HpackHeaderBasic {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: (HpackHeaderName, &str)) -> Result<Self, Self::Error> {
    Ok(match from.0 {
      HpackHeaderName::Authority => HpackHeaderBasic::Authority,
      HpackHeaderName::Field => HpackHeaderBasic::Field,
      HpackHeaderName::Method => {
        HpackHeaderBasic::Method(Method::from_canonical_bytes(from.1.as_bytes())?)
      }
      HpackHeaderName::Path => HpackHeaderBasic::Path,
      HpackHeaderName::Protocol => {
        HpackHeaderBasic::Protocol(Protocol::from_canonical_bytes(from.1.as_bytes())?)
      }
      HpackHeaderName::Scheme => HpackHeaderBasic::Scheme,
      HpackHeaderName::StatusCode => HpackHeaderBasic::StatusCode(from.1.try_into()?),
    })
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HpackHeaderName {
  Authority,
  Field,
  Method,
  Path,
  Protocol,
  Scheme,
  StatusCode,
}

impl HpackHeaderName {
  pub(crate) const fn new(name: &[u8]) -> crate::Result<Self> {
    Ok(match name {
      b":authority" => Self::Authority,
      b":method" => Self::Method,
      b":path" => Self::Path,
      b":protocol" => Self::Protocol,
      b":scheme" => Self::Scheme,
      b":status" => Self::StatusCode,
      [b':', ..] => return Err(protocol_err(Http2Error::UnexpectedPreFixedHeaderName)),
      _ => Self::Field,
    })
  }

  pub(crate) const fn is_field(self) -> bool {
    matches!(self, HpackHeaderName::Field)
  }
}

#[cfg(test)]
mod tests {
  use crate::http2::hpack_headers::HpackHeaders;

  #[test]
  fn len_surplus() {
    let mut h = HpackHeaders::<u32>::new(100);
    h.push_front(false, 0, "abcdefghij", ["0123456789"], |_| {}).unwrap();
    assert_eq!(h.headers_len(), 1);
    h.push_front(false, 0, "abcdefghij", ["0123456789"], |_| {}).unwrap();
    assert_eq!(h.headers_len(), 1);
  }
}
