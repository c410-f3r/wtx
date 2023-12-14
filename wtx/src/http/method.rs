/// HTTP method
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Method {
  /// Connect
  Connect,
  /// Delete
  Delete,
  /// Get
  Get,
  /// Head
  Head,
  /// Options
  Options,
  /// Patch
  Patch,
  /// Post
  Post,
  /// Put
  Put,
  /// Trace
  Trace,
}

impl TryFrom<&[u8]> for Method {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &[u8]) -> Result<Self, Self::Error> {
    Ok(match from {
      b"CONNECT" => Self::Connect,
      b"DELETE" => Self::Delete,
      b"GET" => Self::Get,
      b"HEAD" => Self::Head,
      b"OPTIONS" => Self::Options,
      b"PATCH" => Self::Patch,
      b"POST" => Self::Post,
      b"PUT" => Self::Put,
      b"TRACE" => Self::Trace,
      _ => return Err(crate::Error::UnexpectedHttpMethod),
    })
  }
}
