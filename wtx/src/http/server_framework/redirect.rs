use crate::http::{
  Header, Headers, KnownHeaderName, ReqResBuffer, ReqResDataMut, Request, StatusCode,
  server_framework::ResFinalizer,
};

/// Redirects a request to another location.
#[derive(Debug)]
pub struct Redirect {
  status_code: StatusCode,
  uri: &'static str,
}

impl Redirect {
  const fn new(status_code: StatusCode, uri: &'static str) -> Self {
    Self { status_code, uri }
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/308>
  #[inline]
  pub const fn permanent(uri: &'static str) -> Self {
    Self::new(StatusCode::PermanentRedirect, uri)
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/308>
  #[inline]
  pub fn permanent_raw(headers: &mut Headers, uri: &str) -> crate::Result<StatusCode> {
    Self::push_headers(headers, uri)?;
    Ok(StatusCode::PermanentRedirect)
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/307>
  #[inline]
  pub const fn temporary(uri: &'static str) -> Self {
    Self::new(StatusCode::TemporaryRedirect, uri)
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/307>
  #[inline]
  pub fn temporary_raw(headers: &mut Headers, uri: &str) -> crate::Result<StatusCode> {
    Self::push_headers(headers, uri)?;
    Ok(StatusCode::TemporaryRedirect)
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/303>
  #[inline]
  pub const fn to(uri: &'static str) -> Self {
    Self::new(StatusCode::SeeOther, uri)
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/303>
  #[inline]
  pub fn to_raw(headers: &mut Headers, uri: &str) -> crate::Result<StatusCode> {
    Self::push_headers(headers, uri)?;
    Ok(StatusCode::SeeOther)
  }

  fn push_headers(headers: &mut Headers, uri: &str) -> crate::Result<()> {
    headers.push_from_iter(Header::from_name_and_value(KnownHeaderName::Location.into(), [uri]))?;
    Ok(())
  }
}

impl<E> ResFinalizer<E> for Redirect
where
  E: From<crate::Error>,
{
  #[inline]
  fn finalize_response(self, req: &mut Request<ReqResBuffer>) -> Result<StatusCode, E> {
    req.clear();
    Self::push_headers(req.rrd.headers_mut(), self.uri)?;
    Ok(self.status_code)
  }
}
