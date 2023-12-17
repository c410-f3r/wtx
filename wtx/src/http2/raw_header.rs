use crate::{
  http::{Method, StatusCode},
  misc::_atoi,
};

#[derive(Debug, Eq, PartialEq)]
pub enum RawHeader<'data> {
  Authority(&'data [u8]),
  Field { name: &'data [u8], value: &'data [u8] },
  Method(Method),
  Path(&'data [u8]),
  Protocol(&'data [u8]),
  Scheme(&'data [u8]),
  Status(StatusCode),
}

impl<'data> RawHeader<'data> {
  pub fn new(name: &'data [u8], value: &'data [u8]) -> crate::Result<Self> {
    let [name_first, name_rest @ ..] = name else {
      return Err(crate::Error::UnexpectedEndOfStream);
    };
    Ok(if name_first == &b':' {
      match name_rest {
        b"authority" => Self::Authority(value),
        b"method" => Self::Method(Method::try_from(value)?),
        b"path" => Self::Path(value),
        b"protocol" => Self::Protocol(value),
        b"scheme" => Self::Scheme(value),
        b"status" => Self::Status(StatusCode::try_from(_atoi::<u16>(value)?)?),
        _ => return Err(crate::Error::UnexpectedPreFixedHeaderName),
      }
    } else {
      Self::Field { name, value }
    })
  }
}
