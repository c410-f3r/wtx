use crate::http::{Headers, HttpError, KnownHeaderName, Method, Mime};

pub(crate) fn check_header_and_method<E>(
  mime: Mime,
  req_headers: &Headers,
  req_method: Method,
  user_method: Method,
) -> Result<&str, E>
where
  E: From<crate::Error>,
{
  if req_method != user_method {
    return Err(E::from(crate::Error::from(HttpError::UnexpectedHttpMethod {
      expected: user_method,
    })));
  }
  let header = req_headers
    .get_by_name(KnownHeaderName::ContentType.into())
    .ok_or(crate::Error::from(HttpError::MissingHeader(KnownHeaderName::ContentType)))?;
  let mime_str = mime.as_str();
  let after_mime_opt = header.value.split_at_checked(mime_str.len()).and_then(|(lhs, rhs)| {
    if lhs != mime_str {
      return None;
    }
    Some(rhs)
  });
  let Some(after_mime) = after_mime_opt else {
    return Err(E::from(crate::Error::from(HttpError::UnexpectedContentType { expected: mime })));
  };
  Ok(after_mime)
}

pub(crate) fn check_method<E>(expected: Method, received: Method) -> Result<(), E>
where
  E: From<crate::Error>,
{
  if expected != received {
    return Err(E::from(crate::Error::from(HttpError::UnexpectedHttpMethod { expected })));
  }
  Ok(())
}
