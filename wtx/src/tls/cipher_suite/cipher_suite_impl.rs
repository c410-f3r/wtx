#[derive(Debug)]
pub(crate) enum CipherSuiteImpl<A, B, C> {
  Aes128GcmSha256(A),
  Aes256GcmSha384(B),
  Chacha20Poly1305Sha256(C),
}
