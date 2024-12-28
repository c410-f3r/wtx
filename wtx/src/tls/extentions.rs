pub enum CertificateExtension<'a> {
  StatusRequest(Unimplemented<'a>),
  SignedCertificateTimestamp(Unimplemented<'a>),
}

pub enum CertificateRequestExtension<'a> {
  StatusRequest(Unimplemented<'a>),
  SignatureAlgorithms(SignatureAlgorithms<19>),
  SignedCertificateTimestamp(Unimplemented<'a>),
  CertificateAuthorities(Unimplemented<'a>),
  OidFilters(Unimplemented<'a>),
  SignatureAlgorithmsCert(Unimplemented<'a>),
}

pub enum ClientHelloExtension<'a> {
  ServerName(ServerNameList<'a, 1>),
  SupportedVersions(SupportedVersionsClientHello<16>),
  SignatureAlgorithms(SignatureAlgorithms<19>),
  SupportedGroups(SupportedGroups<16>),
  KeyShare(KeyShareClientHello<'a, 1>),
  PreSharedKey(PreSharedKeyClientHello<'a, 4>),
  PskKeyExchangeModes(PskKeyExchangeModes<4>),
  SignatureAlgorithmsCert(SignatureAlgorithmsCert<19>),
  MaxFragmentLength(MaxFragmentLength),
  StatusRequest(Unimplemented<'a>),
  UseSrtp(Unimplemented<'a>),
  Heartbeat(Unimplemented<'a>),
  ApplicationLayerProtocolNegotiation(Unimplemented<'a>),
  SignedCertificateTimestamp(Unimplemented<'a>),
  ClientCertificateType(Unimplemented<'a>),
  ServerCertificateType(Unimplemented<'a>),
  Padding(Unimplemented<'a>),
  EarlyData(Unimplemented<'a>),
  Cookie(Unimplemented<'a>),
  CertificateAuthorities(Unimplemented<'a>),
  OidFilters(Unimplemented<'a>),
  PostHandshakeAuth(Unimplemented<'a>),
}

pub enum EncryptedExtensionsExtension<'a> {
  ServerName(ServerNameResponse),
  MaxFragmentLength(MaxFragmentLength),
  SupportedGroups(SupportedGroups<10>),
  UseSrtp(Unimplemented<'a>),
  Heartbeat(Unimplemented<'a>),
  ApplicationLayerProtocolNegotiation(Unimplemented<'a>),
  ClientCertificateType(Unimplemented<'a>),
  ServerCertificateType(Unimplemented<'a>),
  EarlyData(Unimplemented<'a>),
}

pub enum HelloRetryRequestExtension<'a> {
  KeyShare(Unimplemented<'a>),
  Cookie(Unimplemented<'a>),
  SupportedVersions(Unimplemented<'a>),
}

pub enum NewSessionTicketExtension<'a> {
  EarlyData(Unimplemented<'a>),
}

pub enum ServerHelloExtension<'a> {
  KeyShare(KeyShareServerHello<'a>),
  PreSharedKey(PreSharedKeyServerHello),
  Cookie(Unimplemented<'a>), // temporary so we don't trip up on HelloRetryRequests
  SupportedVersions(SupportedVersionsServerHello),
}
