/// Specifies the group or curve used for key exchange mechanisms.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NamedGroup {
  /// Secp256r1
  Secp256r1,
  /// Secp384r1
  Secp384r1,
  /// Secp521r1
  Secp521r1,
  /// X25519
  X25519,
  /// X448
  X448,

  /// Ffdhe2048
  Ffdhe2048,
  /// Ffdhe3072
  Ffdhe3072,
  /// Ffdhe4096
  Ffdhe4096,
  /// Ffdhe6144
  Ffdhe6144,
  /// Ffdhe8192
  Ffdhe8192,
}
