use crate::http::{Method, StatusCode};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HpackHeaderBasic {
  Authority,
  Field,
  Method(Method),
  Path,
  Protocol,
  Scheme,
  Status(StatusCode),
}

impl HpackHeaderBasic {
  pub(crate) const fn len(&self, name: &[u8], value: &[u8]) -> usize {
    match self {
      HpackHeaderBasic::Authority => 10usize.wrapping_add(value.len()).wrapping_add(32),
      HpackHeaderBasic::Field => name.len().wrapping_add(value.len()).wrapping_add(32),
      HpackHeaderBasic::Method(_) => 6usize.wrapping_add(value.len()).wrapping_add(32),
      HpackHeaderBasic::Path => 5usize.wrapping_add(value.len()).wrapping_add(32),
      HpackHeaderBasic::Protocol => 9usize.wrapping_add(value.len()).wrapping_add(32),
      HpackHeaderBasic::Scheme => 7usize.wrapping_add(value.len()).wrapping_add(32),
      HpackHeaderBasic::Status(_) => 7usize.wrapping_add(3).wrapping_add(32),
    }
  }

  pub(crate) const fn name<'name>(&self, name: &'name [u8]) -> &'name [u8] {
    match self {
      HpackHeaderBasic::Authority => b":authority",
      HpackHeaderBasic::Field => name,
      HpackHeaderBasic::Method(_) => b":method",
      HpackHeaderBasic::Path => b":path",
      HpackHeaderBasic::Protocol => b":protocol",
      HpackHeaderBasic::Scheme => b":scheme",
      HpackHeaderBasic::Status(_) => b":status",
    }
  }
}

impl TryFrom<(HpackHeaderName, &[u8])> for HpackHeaderBasic {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: (HpackHeaderName, &[u8])) -> Result<Self, Self::Error> {
    Ok(match from.0 {
      HpackHeaderName::Authority => HpackHeaderBasic::Authority,
      HpackHeaderName::Field => HpackHeaderBasic::Field,
      HpackHeaderName::Method => HpackHeaderBasic::Method(from.1.try_into()?),
      HpackHeaderName::Path => HpackHeaderBasic::Path,
      HpackHeaderName::Protocol => HpackHeaderBasic::Protocol,
      HpackHeaderName::Scheme => HpackHeaderBasic::Scheme,
      HpackHeaderName::Status => HpackHeaderBasic::Status(from.1.try_into()?),
    })
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HpackHeaderName {
  Authority,
  Field,
  Method,
  Path,
  Protocol,
  Scheme,
  Status,
}

impl HpackHeaderName {
  pub(crate) fn new(name: &[u8]) -> crate::Result<Self> {
    Ok(match name {
      b":authority" => Self::Authority,
      b":method" => Self::Method,
      b":path" => Self::Path,
      b":protocol" => Self::Protocol,
      b":scheme" => Self::Scheme,
      b":status" => Self::Status,
      [b':', ..] => return Err(crate::Error::UnexpectedPreFixedHeaderName),
      _ => Self::Field,
    })
  }

  pub(crate) fn is_field(&self) -> bool {
    matches!(self, HpackHeaderName::Field)
  }
}
