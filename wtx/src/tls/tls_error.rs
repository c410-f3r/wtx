/// TLS errror
#[derive(Clone, Copy, Debug)]
pub enum TlsError {
  /// Received an alert record in teh handshake phase
  AbortedHandshake,
  /// Bad Pre Key Share
  BadPreKeyShare,
  /// Diffie–Hellman error
  DiffieHellmanError,
  /// Duplicated Client Hello Parameters
  DuplicatedClientHelloParameters,
  /// Incompatible Certificate Types
  IncompatibleCertificateTypes,
  /// Invalid Alert
  InvalidAlert,
  /// Invalid array
  InvalidArray,
  /// Invalid slice
  InvalidSlice,
  /// Invalid certificate
  InvalidCertificate,
  /// Invalid certificate request
  InvalidCertificateRequest,
  /// Invalid Certificate Type
  InvalidCertificateType,
  /// Invalid certificate verify
  InvalidCertificateVerify,
  /// Invalid Cipher Suite
  InvalidCipherSuite,
  /// Invalid client hello length
  InvalidClientHelloLength,
  /// Invalid cookie
  InvalidCookie,
  /// Invalid Encrypted Extensions
  InvalidEncryptedExtensions,
  /// Invalid client hello
  InvalidExtension,
  /// Invalid Finished Record
  InvalidFinishedRecord,
  /// Invalid Handshake
  InvalidHandshake,
  /// Invalid Legacy Session Id
  InvalidLegacySessionId,
  /// Invalid new session ticket
  InvalidNewSessionTicket,
  /// Invalid Key Share Client Hello
  InvalidKeyShareClientHello,
  /// Invalid Key Share
  InvalidKeyShare,
  /// Invalid Key Share Entry
  InvalidKeyShareEntry,
  /// Invalid key update state
  InvalidKeyUpdateState,
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
  /// Invalid server hello
  InvalidServerHello,
  /// Invalid Legacy Session Id Echo
  InvalidLegacySessionIdEcho,
  /// Invalid Raw Public Key
  InvalidRawPublicKey,
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
  /// Invalid u24 prefix
  InvalidU24Prefix,
  /// Mismatch Extension
  MismatchedExtension,
  /// Missing Key Shares
  MissingKeyShares,
  /// Missing signature algorithms
  MissingSignatureAlgorithms,
  /// Missing `supported_versions`
  MissingSupportedVersions,
  /// Record extrapolates the maximum fragment length
  ReceivedRecordIsTooLarge,
  /// Unknown name type
  UnknownNameType,
  /// Unknown Webpki Signature Scheme
  UnknownWebpkiSignatureScheme,
  /// mTLS is not supported
  UnsupportedMtls,
  /// Secret mismatch
  SecretMismatch,
  /// The server has a set of suites that the client don't support
  ServerHasNoCompatibleAlgorithmTy,
  /// The server has a set of suites that the client don't support
  ServerHasNoCompatibleCypherSuite,
  /// The server has a set of suites that the client don't support
  ServerHasNoCompatibleKeyShare,
  /// The capacity upper bound of `TlsReadBuffer` was extrapolated
  TlsReadBufferOverflow,
  /// Records like `ChangeCipherSpec` are not allowed as an inner type
  UnexpectedAfterHandshakeInnerRecord,
  /// Only an outer `ApplicationData` is allowed after the handshake
  UnexpectedAfterHandshakeOuterRecord,
  /// Unsupported extension
  UnsupportedExtension,
  /// Only TLS 1.3 is supported
  UnsupportedTlsVersion,
}
