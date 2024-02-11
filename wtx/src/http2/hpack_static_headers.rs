use crate::{
  http::{Method, StatusCode},
  http2::HpackHeaderBasic,
};
use core::iter;

/// Mandatory headers of a HTTP/2 request
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct HpackStaticRequestHeaders<'bytes> {
  pub(crate) authority: &'bytes [u8],
  pub(crate) method: Method,
  pub(crate) path: &'bytes [u8],
  pub(crate) protocol: &'bytes [u8],
  pub(crate) scheme: &'bytes [u8],
}

impl<'bytes> HpackStaticRequestHeaders<'bytes> {
  pub(crate) fn iter(&self) -> impl Iterator<Item = (HpackHeaderBasic, &[u8])> {
    let Self { authority, method, path, protocol, scheme } = *self;
    let mandatory = iter::once((HpackHeaderBasic::Method(method), &[][..]));
    let optional = [
      (HpackHeaderBasic::Authority, authority),
      (HpackHeaderBasic::Path, path),
      (HpackHeaderBasic::Protocol, protocol),
      (HpackHeaderBasic::Scheme, scheme),
    ]
    .into_iter()
    .filter(|el| !el.1.is_empty());
    mandatory.chain(optional)
  }
}

/// Mandatory headers of a HTTP/2 response
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct HpackStaticResponseHeaders {
  pub(crate) status: StatusCode,
}

impl HpackStaticResponseHeaders {
  pub(crate) fn iter(&self) -> impl Iterator<Item = (HpackHeaderBasic, &[u8])> {
    let Self { status } = *self;
    iter::once((HpackHeaderBasic::Status(status), &[][..]))
  }
}
