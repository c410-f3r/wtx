/// Type of the Pre Shared Key
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PskTy {
  /// Provisioned outside of TLS
  External,
  /// Provisioned as the resumption master secret of a previous handshake
  Resumption,
}
