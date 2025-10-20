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
  (Aes128GcmAwsLc, &'static aws_lc_rs::aead::Algorithm, &aws_lc_rs::aead::AES_128_GCM)
  (Aes256GcmAwsLc, &'static aws_lc_rs::aead::Algorithm, &aws_lc_rs::aead::AES_256_GCM)
  (Chacha20Poly1305AwsLc, &'static aws_lc_rs::aead::Algorithm, &aws_lc_rs::aead::CHACHA20_POLY1305)

  (Sha256AwsLc, &'static aws_lc_rs::digest::Algorithm, &aws_lc_rs::digest::SHA256)
  (Sha384AwsLc, &'static aws_lc_rs::digest::Algorithm, &aws_lc_rs::digest::SHA384)
  
}

#[cfg(feature = "ring")]
_create! {
  (Aes128GcmRing, &'static ring::aead::Algorithm, &ring::aead::AES_128_GCM)
  (Aes256GcmRing, &'static ring::aead::Algorithm, &ring::aead::AES_256_GCM)
  (Chacha20Poly1305Ring, &'static ring::aead::Algorithm, &ring::aead::CHACHA20_POLY1305)

  (Sha256Ring, &'static ring::digest::Algorithm, &ring::digest::SHA256)
  (Sha384Ring, &'static ring::digest::Algorithm, &ring::digest::SHA384)
}
