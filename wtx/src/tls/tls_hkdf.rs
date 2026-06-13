pub(crate) type TlsHkdf =
  crate::misc::Either<crate::crypto::HkdfSha256Global, crate::crypto::HkdfSha384Global>;
