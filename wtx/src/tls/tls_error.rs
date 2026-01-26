/// TLS errror
#[derive(Debug)]
pub enum TlsError {
  /// Received an alert record in teh handshake phase
  AbortedHandshake,
  /// Bad Pre Key Share
  BadPreKeyShare,
  /// Diffie–Hellman error
  DiffieHellmanError,
  /// Duplicated Client Hello Parameters
  DuplicatedClientHelloParameters,
  /// Invalid array
  InvalidArray,
  /// Invalid slice
  InvalidSlice,
  /// Invalid Cipher Suite
  InvalidCipherSuite,
  /// Invalid client hello
  InvalidExtension,
  /// Invalid client hello length
  InvalidClientHelloLength,
  /// Invalid Handshake
  InvalidHandshake,
  /// Invalid Alert
  InvalidAlert,
  /// Invalid Legacy Session Id
  InvalidLegacySessionId,
  /// Invalid Key Share Client Hello
  InvalidKeyShareClientHello,
  /// Invalid Key Share
  InvalidKeyShare,
  /// Invalid Key Share Entry
  InvalidKeyShareEntry,
  /// Invalid Max Fragment Length
  InvalidMaxFragmentLength,
  /// Invalid protocol version
  InvalidProtocolVersion,
  /// Invalid Psk Key Exchange Modes
  InvalidPskKeyExchangeModes,
  /// Invalid Signature Algorithms
  InvalidSignatureAlgorithms,
  /// Invalid Signature Algorithms Certificate
  InvalidSignatureAlgorithmsCert,
  /// Invalid Signature Scheme
  InvalidSignatureScheme,
  /// Invalid Supported Groups
  InvalidSupportedGroups,
  /// Invalid Supported Versions Of Client Hello
  InvalidSupportedVersions,
  /// Invalid server hello length
  InvalidServerHelloLen,
  /// Invalid Legacy Session Id Echo
  InvalidLegacySessionIdEcho,
  /// Invalid server name
  InvalidServerName,
  /// Invalid server name list
  InvalidServerNameList,
  /// Invalid Offered Psks
  InvalidOfferedPsks,
  /// Invalid u8 prefix
  InvalidU8Prefix,
  /// Invalid u16 prefix
  InvalidU16Prefix,
  /// Mismatch Extension
  MismatchedExtension,
  /// Unsupported extension
  UnsupportedExtension,
  /// Only TLS 1.3 is supported
  UnsupportedTlsVersion,
  /// Missing Key Shares
  MissingKeyShares,
  /// Missing signature algorithms
  MissingSignatureAlgorithms,
  /// Missing supported_versions
  MissingSupportedVersions,
  /// Unknown name type
  UnknownNameType,
  /// Unknown Webpki Signature Scheme
  UnknownWebpkiSignatureScheme,
  /// Secret mismatch
  SecretMismatch,
  /// The server has a set of suites that the client don't support
  ServerNoCompatibleCypherSuite,
  /// The server has a set of suites that the client don't support
  ServerNoCompatibleKeyShare,
}
