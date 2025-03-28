pub(crate) mod get;
pub(crate) mod json;
pub(crate) mod post;
pub(crate) mod web_socket;

use crate::http::{Headers, HttpError, KnownHeaderName, Method, Mime};

#[inline]
fn check_method<E>(expected: Method, received: Method) -> Result<(), E>
where
  E: From<crate::Error>,
{
  if expected != received {
    return Err(E::from(crate::Error::from(HttpError::UnexpectedHttpMethod { expected })));
  }
  Ok(())
}

#[inline]
fn check_json<E>(headers: &Headers, method: Method) -> Result<(), E>
where
  E: From<crate::Error>,
{
  let header = headers
    .get_by_name(KnownHeaderName::ContentType.into())
    .ok_or(crate::Error::from(HttpError::MissingHeader(KnownHeaderName::ContentType)))?;
  if header.value != Mime::ApplicationJson.as_str() {
    return Err(E::from(crate::Error::from(HttpError::UnexpectedContentType)));
  }
  if method != Method::Post {
    return Err(E::from(crate::Error::from(HttpError::UnexpectedHttpMethod {
      expected: Method::Post,
    })));
  }
  Ok(())
}
