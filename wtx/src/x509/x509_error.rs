/// X.509 error
#[derive(Debug)]
pub enum X509Error {
  /// Invalid X.509 Algorithm Identifier
  InvalidAlgorithmIdentifier,
  /// Invalid X.509 Attribute Type and Value pair
  InvalidAttributeTypeAndValue,
  /// Invalid X.509 Certificate
  InvalidCertificate,
  /// Invalid X.509 Extension
  InvalidExtension,
  /// Invalid X.509 Distinguished Name
  InvalidName,
  /// Invalid X.509 Object Identifier
  InvalidObjectIdentifier,
  /// Invalid X.509 Subject Public Key Info
  InvalidSubjectPublicKeyInfo,
  /// Invalid X.509 TBS (To Be Signed) Certificate
  InvalidTbsCertificate,
  /// Invalid X.509 Validity period
  InvalidValidity,
  /// Only version 3 is supported.
  InvalidVersion,
}
