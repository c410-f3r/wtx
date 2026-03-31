use crate::http::{
  Mime, ReqBuilder, ReqResBuffer, Request, StatusCode, server_framework::ResFinalizer,
};

/// Finalizes the response by setting the `content-type` header to `application/json`.
///
/// This type **DOES NOT** automatically serializes data. You must manually encode your payload
/// and write it to the response body before returning this finalizer.
#[derive(Debug)]
pub struct JsonReply(
  /// See [`StatusCode`].
  pub StatusCode,
);

impl<E> ResFinalizer<E> for JsonReply
where
  E: From<crate::Error>,
{
  #[inline]
  fn finalize_response(self, req: &mut Request<ReqResBuffer>) -> Result<StatusCode, E> {
    drop(ReqBuilder::from_req_mut(req).content_type(Mime::ApplicationJson));
    Ok(self.0)
  }
}

impl Default for JsonReply {
  #[inline]
  fn default() -> Self {
    Self(StatusCode::Ok)
  }
}
