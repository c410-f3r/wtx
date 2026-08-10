use crate::{
  net::RoleTy,
  tls::{Alert, ProtocolVersion, protocol::handshake_ty::HandshakeTy},
};

/// TLS errror
#[derive(Clone, Copy, Debug)]
pub enum TlsError {
  /// Received an alert record in teh handshake phase
  AbortedHandshake(Alert),
  /// Peer closed the connection without a graceful stop
  AbruptDisconnect,
  /// Bad Pre Key Share
  BadPreKeyShare,
  /// Bad signature
  BadSignature,
  /// Expected Finished record
  ClientExpectedFinished,
  /// Digest Check Failed
  DigestCheckFailed,
  /// Diffie–Hellman error
  DiffieHellmanError,
  /// Duplicated Certificate Request Parameters
  DuplicatedCertificateRequestParameters,
  /// Duplicated Client Hello Parameters
  DuplicatedClientHelloParameters,
  /// Duplicated Encrypted Extensions Parameters
  DuplicatedEncryptedExtensionsParameters,
  /// Duplicated Key Shares
  DuplicatedKeyShares,
  /// Empty Certificate Authorities
  EmptyCertificateAuthorities,
  /// Invalid Negotiated ALPN
  EmptyNegotiatedAlpnClient,
  /// Invalid Negotiated ALPN
  EmptyNegotiatedAlpnServer,
  /// Empty New Session Ticket
  EmptyNewSessionTicket,
  /// Empty set of certificates
  EmptySetOfCertificates,
  /// Trailing Data In Handshake
  ExcessHandshakeData(RoleTy),
  /// Incompatible ALPN
  IncompatibleAlpn,
  /// Incomplete Handshake
  IncompleteHandshake,
  /// Incompatible Certificate Types
  IncompatibleCertificateTypes,
  /// Invalid AES data
  InvalidAesData,
  /// Invalid Alert
  InvalidAlert,
  /// Invalid array
  InvalidArray,
  /// Invalid slice
  InvalidSlice,
  /// Invalid certificate
  InvalidCertificate,
  /// Invalid certificate authorities
  InvalidCertificateAuthorities,
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
  /// Invalid extension
  InvalidExtension,
  /// Invalid extension type
  InvalidExtensionTy,
  /// Invalid Finished Record
  InvalidFinishedRecord,
  /// Invalid Handshake Length
  InvalidHandshakeLen,
  /// Invalid Handshake Type
  InvalidHandshakeTy,
  /// Invalid Legacy Compression Method (Server)
  InvalidLegacyCompressionMethod,
  /// Invalid Legacy Compression Methods (Client)
  InvalidLegacyCompressionMethods,
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
  /// Invalid Negotiated Max Fragment Length
  InvalidNegotiatedMaxFragmentLength,
  /// Invalid Negotiated Server Name
  InvalidNegotiatedServerName,
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
  /// Invalid Psk Key Exchange Mode
  InvalidPskKeyExchangeMode,
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
  /// Invalid X.509
  InvalidX509,
  /// For example, public key is PSS but signature is RSAE
  MismatchedCertificatePkAndSignature,
  /// Mismatch Extension
  MismatchedExtension,
  /// Invalid Negotiated ALPN
  MismatchedNegotiatedAlpnClient,
  /// Invalid Negotiated ALPN
  MismatchedNegotiatedAlpnServer,
  /// Missing Digital Signature in Key Usage
  MissingDigitalSignatureInKeyUsage,
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
  /// No leaf certificate in chain
  NoLeafCertInChain,
  /// Pre Handshake Decoder Error
  PreHandshakeDecError,
  /// Post Handshake Decoder Error
  PostHandshakeDecError(HandshakeTy),
  /// Record extrapolates the maximum fragment length
  ReceivedRecordIsTooLarge,
  /// Too many key updates
  TooManyKeyUpdates,
  /// Too many warning alerts
  TooManyWarningAlerts,
  /// Trailing data in extension
  TrailingDataInExtension,
  /// Record was supposed to be encrypted
  UnencryptedRecord,
  /// Unknown name type
  UnknownNameType,
  /// Unknowns overflow
  UnknownsOverflow,
  /// Unoffered Extension
  UnofferedExtension,
  /// Unknown Key Update Request
  UnknownKeyUpdateRequest,
  /// Unknown Signature Scheme
  UnknownSignatureScheme,
  /// Unknown Webpki Signature Scheme
  UnknownWebpkiSignatureScheme,
  /// Secret mismatch
  SecretMismatch,
  /// The server has a set of suites that the client don't support
  ServerHasNoCompatibleSignatureScheme,
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
  /// Unsupported Cipher Suite
  UnsupportedCipherSuite,
  /// Unsupported extension
  UnsupportedExtension,
  /// mTLS is not supported
  UnsupportedMtls,
  /// Only TLS 1.2 is supported due to legacy reasons
  UnsupportedRecTlsVersion(ProtocolVersion),
  /// Unsupported Sign Algorithm
  UnsupportedSignAlgorithm,
  /// Only TLS 1.3 is supported
  UnsupportedTlsVersion(Option<ProtocolVersion>),
  /// Unknown handshake type
  UnknownHandshakeTy(u8),
  /// Unknown Named Group
  UnknownNamedGroup,
  /// Unknown Protocol Version
  UnknownProtocolVersion,
  /// Unknown record content type
  UnknownRecordContentType,
  /// Wrong alert
  WrongAlert,
}
