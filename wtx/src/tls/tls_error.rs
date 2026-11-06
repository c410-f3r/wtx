use crate::{
  net::RoleTy,
  tls::{Alert, ProtocolVersion, handshake_ty::HandshakeTy},
};

/// TLS errror
#[derive(Clone, Copy, Debug)]
pub enum TlsError {
  /// Received an alert record in teh handshake phase
  AbortedHandshake(Alert),
  /// Peer closed the connection without a graceful stop
  AbruptDisconnect,
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
  /// Empty inner plaintext of a protected record
  EmptyPlaintext,
  /// Trailing Data In Handshake
  ExcessHandshakeData(RoleTy),
  /// Incomplete Handshake
  IncompleteHandshake,
  /// Invalid AES data
  InvalidAesData,
  /// Invalid Alert
  InvalidAlert,
  /// Invalid array
  InvalidArray,
  /// Invalid certificate
  InvalidCertificate,
  /// Invalid certificate authorities
  InvalidCertificateAuthorities,
  /// Invalid certificate request
  InvalidCertificateRequest,
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
  /// Invalid Key Share Entry
  InvalidKeyShareEntry,
  /// Invalid Max Fragment Length
  InvalidMaxFragmentLength,
  /// Invalid Negotiated Max Fragment Length
  InvalidNegotiatedMaxFragmentLength,
  /// Invalid Negotiated Server Name
  InvalidNegotiatedServerName,
  /// Invalid Supported Versions Of Client Hello
  InvalidSupportedVersions,
  /// Invalid server hello
  InvalidServerHello,
  /// Invalid Psk Key Exchange Mode
  InvalidPskKeyExchangeMode,
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
  /// Secret mismatch
  SecretMismatch,
  /// The server has a set of suites that the client don't support
  ServerHasNoCompatibleSignatureScheme,
  /// The server has a set of suites that the client don't support
  ServerHasNoCompatibleCypherSuite,
  /// The server has a set of suites that the client don't support
  ServerHasNoCompatibleKeyShare,
  /// Usage of deprecated PKCS1
  UsageOfDeprecatedPKCS1,
  /// Records like `ChangeCipherSpec` are not allowed as an inner type
  UnexpectedAfterHandshakeInnerRecord,
  /// Only an outer `ApplicationData` is allowed after the handshake
  UnexpectedAfterHandshakeOuterRecord,
  /// Unsupported Cipher Suite
  UnsupportedCipherSuite,
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
  UnknownRecordContentType(u8),
  /// Wrong alert
  WrongAlert,
}
