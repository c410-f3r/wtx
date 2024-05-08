pub(crate) fn _host_from_args() -> String {
  std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:9000".to_owned())
}

pub(crate) fn _uri_from_args() -> String {
  std::env::args().nth(1).unwrap_or_else(|| "http://127.0.0.1:9000".to_owned())
}
