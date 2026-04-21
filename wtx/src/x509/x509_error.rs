/// X.509 error
#[derive(Debug)]
pub enum X509Error {
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
  /// Invalid Certificate Version (only v3 supported)
  InvalidCertificateVersion,
  /// Invalid Certificate Revocation List
  InvalidCrl,
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
  /// Invalid RSASSA-PSS Parameters
  InvalidRsassaPssParams,
  /// Invalid Subject Alternative NAme
  InvalidSan,
  /// A sequence of DER bytes can not represent a serial number.
  InvalidSerialNumberBytes,
  /// Invalid Subject Public Key Info
  InvalidSubjectPublicKeyInfo,
  /// Invalid TBS (To Be Signed) Certificate
  InvalidTbsCertificate,
  /// Invalid TBS Certificate List
  InvalidTbsCertList,
  /// Invalid TBS Certificate List Version
  InvalidTbsCertListVersion,
  /// Invalid time
  InvalidTime,
  /// Invalid Validity period
  InvalidValidity,
}
