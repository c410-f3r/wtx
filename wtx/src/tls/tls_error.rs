use crate::tls::{Alert, ProtocolVersion};

/// TLS errror
#[derive(Clone, Copy, Debug)]
pub enum TlsError {
  /// Received an alert record in teh handshake phase
  AbortedHandshake(Alert),
  /// Peer closed the connection without a graceful stop
  AbruptDisconnect,
  /// Bad Pre Key Share
  BadPreKeyShare,
  /// Diffie–Hellman error
  DiffieHellmanError,
  /// Duplicated Client Hello Parameters
  DuplicatedClientHelloParameters,
  /// Empty set of certificates
  EmptySetOfCertificates,
  /// Incompatible ALPN
  IncompatibleAlpn,
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
  /// Invalid Legacy Compression Method
  InvalidLegacyCompressionMethod,
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
  /// Missing supported groups
  MissingSupportedGroups,
  /// Missing `supported_versions`
  MissingSupportedVersions,
  /// No certificate received
  NoCertificate,
  /// Record extrapolates the maximum fragment length
  ReceivedRecordIsTooLarge,
  /// Record was supposed to be encrypted
  UnencryptedRecord,
  /// Unknown name type
  UnknownNameType,
  /// Unknown Webpki Signature Scheme
  UnknownWebpkiSignatureScheme,
  /// Unsupported Cipher Suite
  UnsupportedCipherSuite,
  /// mTLS is not supported
  UnsupportedMtls,
  /// Can not receive certificate records once a PSK was accepted
  CertRecordInAcceptedPsk,
  /// Secret mismatch
  SecretMismatch,
  /// The server has a set of suites that the client don't support
  ServerHasNoCompatibleAlgorithmTy,
  /// The server has a set of suites that the client don't support
  ServerHasNoCompatibleAlgorithmTyForCert,
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
  UnsupportedTlsVersion(Option<ProtocolVersion>),
  /// Only TLS 1.2 is supported due to legacy reasons
  UnsupportedRecTlsVersion(ProtocolVersion),
}
