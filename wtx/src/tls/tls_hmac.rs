pub(crate) type TlsHmac =
  crate::misc::Either<crate::crypto::HmacSha256Global, crate::crypto::HmacSha384Global>;
