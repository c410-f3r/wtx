/// X.509 chain validation error
#[derive(Debug)]
pub enum X509CvError {
  /// Authority Key Identifier extension must not be marked critical.
  AuthorityKeyIdentifierMustNotBeCritical,
  /// Certificate contains duplicate extensions.
  CertCanNotHaveDuplicateExtensions,
  /// Certificate signature algorithm does not match expected algorithm.
  CertificateAlgorithmMismatch,
  /// Certificate contains an unknown critical extension.
  CertsMustNotHaveCriticalUnknownExtensions,
  /// No valid certification path found to a trust anchor.
  ChainValidationDidNotFindPath,
  /// CRL Number extension must not be marked critical.
  CrlNumberMustNotBeCritical,
  /// Subject name does not match permitted name constraints.
  DoesNotHaveMatchedConstraints,
  /// End Entity certificate cannot have a critical Extended Key Usage.
  EeCanNotHaveACriticalEku,
  /// End Entity certificate must have an Extended Key Usage extension.
  EeMustHaveEku,
  /// Extended Key Usage cannot contain the anyExtendedKeyUsage OID.
  EkuCanNotBeAny,
  /// Extended Key Usage sequence cannot be empty.
  EkuCanNotBeEmpty,
  /// Extended Key Usage does not match the required policy usage.
  EkuMismatch,
  /// Certification path length exceeds the maximum allowed depth.
  ExceedDepth,
  /// Subject name matches an excluded name constraint subtree.
  HasExcludedCerts,
  /// Certificate or CRL validity period has expired.
  HasExpiredCerts,
  /// Key Usage bits are incompatible with the certificate role.
  HasIncompatibleKeyUsage,
  /// Signature algorithm or key is incompatible for verification.
  HasIncompatibleSignature,
  /// No matching trust anchor found for the issuer.
  HasNotTrustAnchor,
  /// Certificate serial number found in a revocation list.
  HasRevokedCerts,
  /// Issuing CAs must have a non-empty subject name.
  IcasMustHaveASubjectSequence,
  /// Issuing CAs must have critical Basic Constraints.
  IcasMustHaveCriticalBasicConstraints,
  /// Issuing CAs must have a Subject Key Identifier.
  IcasMustHaveSki,
  /// Authority Key Identifier extension is malformed.
  InvalidAuthorityKeyIdentifier,
  /// Name Constraints extension is malformed or invalid.
  InvalidNameConstraints,
  /// IP address in subject name cannot be hex formatted.
  IpCanNotBeHex,
  /// CRL is missing the mandatory CRL Number extension.
  MissingCrlNumber,
  /// Name Constraints extension must be marked critical.
  NameConstraintsMustBeCritical,
  /// Name constraints processing exceeded internal limits.
  NameConstraintsOverflow,
  /// Policy Constraints extension must be marked critical.
  PolicyConstraintMustBeCritical,
  /// Root CA must have Subject and Authority Key Identifiers.
  RootCasMustHaveKeyIdentifiers,
  /// Root CA Authority Key Identifier must match Subject Key Identifier.
  RootCasMustHaveMatchingAkiAndSki,
  /// Subject Alternative Name must be critical if subject is empty.
  SanMustBeCritical,
  /// Cryptographic signature verification failed.
  SignatureMismatch,
  /// Subject Key Identifier extension must not be marked critical.
  SubjectKeyIdentifierMustNotBeCritical,
  /// Subject name type or format is unrecognized.
  UnknownSubjectName,
}
