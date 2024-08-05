//! WTX instances

/// Host from arguments
#[inline]
pub fn host_from_args() -> String {
  std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:9000".to_owned())
}

/// Uri from arguments
#[inline]
pub fn uri_from_args() -> String {
  std::env::args().nth(1).unwrap_or_else(|| "http://127.0.0.1:9000".to_owned())
}
