create_enum! {
  /// Specifies the group or curve used for key exchange mechanisms.
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum AgreementAlgorithmTy<u16> {
    /// Secp256r1
    Secp256r1 = (23),
    /// Secp384r1
    Secp384r1 = (24),
    /// X25519
    X25519 = (29),
  }
}
