use crate::misc::UriRef;

/// When sending a request, it is possible to choose the origin of the URI.
#[derive(Clone, Copy, Debug)]
pub enum ReqUri<'uri> {
  /// URI has to be retrieved from the request data. Useful when the URI needs to be constructed
  /// on the fly
  Data,
  /// URI has to be retrieved from this variant. Useful then the URI is known before hand.
  Param(&'uri UriRef<'uri>),
}

impl<'uri> From<&'uri UriRef<'uri>> for ReqUri<'uri> {
  #[inline]
  fn from(from: &'uri UriRef<'uri>) -> Self {
    Self::Param(from)
  }
}
