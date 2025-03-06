macro_rules! create {
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

#[cfg(feature = "tls-aws-lc")]
create! {
  (Aes128GcmAwsLc, &'static aws_lc_rs::aead::Algorithm, &aws_lc_rs::aead::AES_128_GCM)
  (Aes256GcmAwsLc, &'static aws_lc_rs::aead::Algorithm, &aws_lc_rs::aead::AES_256_GCM)

  (Sha256AwsLc, &'static aws_lc_rs::digest::Algorithm, &aws_lc_rs::digest::SHA256)
  (Sha384AwsLc, &'static aws_lc_rs::digest::Algorithm, &aws_lc_rs::digest::SHA384)
}

#[cfg(feature = "tls-ring")]
create! {
  (Aes128GcmRing, &'static ring::aead::Algorithm, &ring::aead::AES_128_GCM)
  (Aes256GcmRing, &'static ring::aead::Algorithm, &ring::aead::AES_256_GCM)

  (Sha256Ring, &'static ring::digest::Algorithm, &ring::digest::SHA256)
  (Sha384Ring, &'static ring::digest::Algorithm, &ring::digest::SHA384)
}
