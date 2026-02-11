/// Crypto error
#[derive(Clone, Copy, Debug)]
pub enum CryptoError {
  /// Opaque error originated from `Hkdf::expand`
  HkdfExpandError,
  /// Opaque error originated from `Hkdf::from_prk`
  HkdfFromPrkError,
  /// Opaque error originated from AES operations
  InvalidAesData,
  /// Opaque error originated from AES-128 operations
  InvalidAes128GcmData,
  /// Opaque error originated from AES-256 operations
  InvalidAes256GcmData,
  /// Opaque error originated from `Chacha20Poly1305` operations
  InvalidChacha20Poly1305Data,
}
