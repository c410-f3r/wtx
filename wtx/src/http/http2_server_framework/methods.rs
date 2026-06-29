pub(crate) mod delete;
pub(crate) mod get;
pub(crate) mod json;
pub(crate) mod patch;
pub(crate) mod post;
pub(crate) mod put;
pub(crate) mod web_socket;

use crate::http::{HttpError, Method};

fn check_method<E>(expected: Method, received: Method) -> Result<(), E>
where
  E: From<crate::Error>,
{
  if expected != received {
    return Err(E::from(crate::Error::from(HttpError::UnexpectedHttpMethod { expected })));
  }
  Ok(())
}
