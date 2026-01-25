macro_rules! _create {
  ($(($name:ident, $ty:ty, $value:expr))*) => {
    $(
      /// A wrapper used to generalize third-party dependencies. Doesn't contain any actual code.
      pub struct $name(pub(crate) $ty);

      impl $name {
        /// New instance
        #[inline]
        pub const fn new() -> Self {
          Self($value)
        }
      }
    )*
  };
}

#[cfg(feature = "aws-lc-rs")]
_create! {
  (Aes128GcmAwsLcRs, aws_lc_rs::aead::Algorithm, aws_lc_rs::aead::AES_128_GCM)
  (Aes256GcmAwsLcRs, aws_lc_rs::aead::Algorithm, aws_lc_rs::aead::AES_256_GCM)
  (Chacha20Poly1305AwsLcRs, aws_lc_rs::aead::Algorithm, aws_lc_rs::aead::CHACHA20_POLY1305)

  (Sha256AwsLcRs, aws_lc_rs::digest::Algorithm, aws_lc_rs::digest::SHA256)
  (Sha384AwsLcRs, aws_lc_rs::digest::Algorithm, aws_lc_rs::digest::SHA384)

  (P256AwsLcRs, aws_lc_rs::agreement::Algorithm, aws_lc_rs::agreement::ECDH_P256)
  (P384AwsLcRs, aws_lc_rs::agreement::Algorithm, aws_lc_rs::agreement::ECDH_P384)
  (X25519AwsLcRs, aws_lc_rs::agreement::Algorithm, aws_lc_rs::agreement::X25519)
}
