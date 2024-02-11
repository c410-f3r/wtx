create_enum! {
  #[derive(Clone, Copy, Debug, Default, PartialEq)]
  /// HTTP version
  pub enum Version<u8> {
    /// HTTP/1
    Http1 = (0),
    /// HTTP/1.1
    Http1_1 = (1),
    /// HTTP/2
    #[default]
    Http2 = (2),
  }
}
