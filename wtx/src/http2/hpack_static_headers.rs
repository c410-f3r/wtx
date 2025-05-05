use crate::{
  http::{Method, Protocol, StatusCode},
  http2::hpack_header::HpackHeaderBasic,
};

/// Mandatory headers of a HTTP/2 request
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HpackStaticRequestHeaders<'bytes> {
  pub(crate) authority: &'bytes str,
  pub(crate) method: Option<Method>,
  pub(crate) path: &'bytes str,
  pub(crate) protocol: Option<Protocol>,
  pub(crate) scheme: &'bytes str,
}

impl HpackStaticRequestHeaders<'_> {
  pub(crate) const EMPTY: Self =
    Self { authority: "", method: None, path: "", protocol: None, scheme: "" };

  pub(crate) fn iter(&self) -> impl Iterator<Item = (HpackHeaderBasic, &str)> {
    let Self { authority, method, path, protocol, scheme } = *self;
    let enums = [
      method.map(|el| (HpackHeaderBasic::Method(el), "")),
      protocol.map(|el| (HpackHeaderBasic::Protocol(el), "")),
    ]
    .into_iter()
    .flatten();
    let uri = [
      (HpackHeaderBasic::Authority, authority),
      (HpackHeaderBasic::Path, path),
      (HpackHeaderBasic::Scheme, scheme),
    ]
    .into_iter()
    .filter(|el| !el.1.is_empty());
    enums.chain(uri)
  }
}

/// Mandatory headers of a HTTP/2 response
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HpackStaticResponseHeaders {
  pub(crate) status_code: Option<StatusCode>,
}

impl HpackStaticResponseHeaders {
  pub(crate) const EMPTY: Self = Self { status_code: None };

  pub(crate) fn iter(&self) -> impl Iterator<Item = (HpackHeaderBasic, &str)> {
    let Self { status_code } = *self;
    status_code.map(|el| (HpackHeaderBasic::StatusCode(el), "")).into_iter()
  }
}
