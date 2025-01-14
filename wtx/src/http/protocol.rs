create_enum! {
  /// Specifies a tunneling protocol
  #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
  pub enum Protocol<u8> {
    /// WebSocket
    WebSocket = (0, "websocket")
  }
}
