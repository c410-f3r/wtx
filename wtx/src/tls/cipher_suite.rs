_create_enum! {
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub(crate) enum CipherSuite<u16> {
    TlsAes128GcmSha256 = (0x1301),
    TlsAes256GcmSha384 = (0x1302),
    TlsChacha20Poly1305Sha256 = (0x1303),
    TlsAes128CcmSha256 = (0x1304),
    TlsAes128Ccm8Sha256 = (0x1305),
    TlsPskAes128GcmSha256 = (0x00A8),
  }
}
