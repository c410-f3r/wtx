#[derive(Debug)]
pub enum CryptoError {
  /// Opaque error originated from `Hkdf::expand`
  HkdfExpandError,
  /// Opaque error originated from `Hkdf::from_prk`
  HkdfFromPrkError,
}
