/// A TLS handshake can follow different paths depending on the negotiated parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HandshakePath {
  /// 1-RTT
  Resumed,
  /// 1-RTT when the remote party asked for a new hello
  ResumedWithHelloRetryRequest,
  /// Normal handshake when the remote party asked for a new hello
  FullWithHelloRetryRequest,
  /// Normal handshake
  Full,
}
