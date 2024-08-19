_create_enum! {
  /// How endpoints should communicate, which is similar but not equal to an URI scheme.
  #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
  pub enum Protocol<u8> {
    /// Http
    Http = (0, "http"),
    /// Https
    Https = (1, "https"),
    /// WebSocket
    WebSocket = (2, "websocket")
  }
}
