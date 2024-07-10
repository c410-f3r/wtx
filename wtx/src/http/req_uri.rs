use crate::misc::UriRef;

/// When sending a request, it is possible to choose the origin of the URI.
#[derive(Debug)]
pub enum ReqUri<'uri> {
  /// URI has to be retrieved from the request data.
  Data,
  /// URI has to be retrieved from the parameter of this variant,
  Param(&'uri UriRef<'uri>),
}
