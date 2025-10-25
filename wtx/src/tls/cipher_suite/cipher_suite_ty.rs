create_enum! {
  /// Refers a concrete cipher suite implementation.
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum CipherSuiteTy<u16> {
    /// TlsAes128GcmSha256
    Aes128GcmSha256 = (0x1301),
    /// TlsAes256GcmSha384
    Aes256GcmSha384 = (0x1302),
    /// TlsChacha20Poly1305Sha256
    Chacha20Poly1305Sha256 = (0x1303),
  }
}
