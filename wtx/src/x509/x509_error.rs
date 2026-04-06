/// X.509 error
#[derive(Debug)]
pub enum X509Error {
  /// Could not not find a path with the given intermediates and trusted chains
  ChainValidationDidNotFindPath,
  /// Path is too long to continue validating a chain
  ChainValidationExceedDepth,
  /// A certificate can not be used for path validation because it is expired.
  ChainValidationHasExpiredCerts,
  /// The signature of a parent certificate did not match the signature of a child certificate
  ChainValidationSignatureMismatch,
  /// Invalid Access Description
  InvalidAccessDescription,
  /// Invalid Algorithm Identifier
  InvalidAlgorithmIdentifier,
  /// Invalid Attribute
  InvalidAttribute,
  /// Invalid Attribute Type and Value pair
  InvalidAttributeTypeAndValue,
  /// Invalid Certificate
  InvalidCertificate,
  /// Invalid Certificate List
  InvalidCertificateList,
  /// Invalid Extended Key Usage
  InvalidExtendedKeyUsage,
  /// Invalid Extension
  InvalidExtension,
  /// Invalid Extensions
  InvalidExtensions(u8),
  /// Invalid Authority Key Identifier extension
  InvalidExtensionAuthorityKeyIdentifier,
  /// Invalid Basic Constraints extension
  InvalidExtensionBasicConstraint,
  /// Invalid Certificate Policies extension
  InvalidExtensionCertificatePolicies,
  /// Invalid CRL Distribution Points extension
  InvalidExtensionCrlDistributionPoints,
  /// Invalid Issuing Distribution Point extension
  InvalidExtensionIssuingDistributionPoint,
  /// Invalid Key Usage extension
  InvalidExtensionKeyUsage,
  /// Invalid Name Constraints extension
  InvalidExtensionNameConstraints,
  /// Invalid Policy Constraints extension
  InvalidExtensionPolicyConstraints,
  /// Invalid Policy Mappings extension
  InvalidExtensionPolicyMappings,
  /// Invalid Reason Code extension
  InvalidExtensionReasonCode,
  /// Invalid General Name
  InvalidGeneralName,
  /// Invalid General Subtree
  InvalidGeneralSubtree,
  /// Invalid Ip Address Representation,
  InvalidIpAddressRepresentation,
  /// Invalid Key Identifier
  InvalidKeyIdentifier,
  /// Invalid Object Identifier
  InvalidObjectIdentifier,
  /// Invalid Reason Flags
  InvalidReasonFlags,
  /// Invalid Revoked Certificate
  InvalidRevokedCertificate,
  /// Invalid Subject Public Key Info
  InvalidSubjectPublicKeyInfo,
  /// Invalid TBS (To Be Signed) Certificate
  InvalidTbsCertificate,
  /// Invalid TBS Certificate List
  InvalidTbsCertList,
  /// Invalid time
  InvalidTime,
  /// Invalid Validity period
  InvalidValidity,
  /// Invalid Version (only v3 supported)
  InvalidVersion,
  /// A Subject Name is not included in the list of extensions nor in the Rdn Sequence.
  UnknownSubjectName,
}
