/// ASN.1 Error
#[derive(Debug)]
pub enum Asn1Error {
  /// Invalid ASN.1 Bit String
  InvalidBitString,
  /// Invalid ASN.1 Boolean
  InvalidBoolean,
  /// Invalid ASN.1 Integer
  InvalidInteger,
  /// Invalid ASN.1 length
  InvalidLen,
  /// Invalid ASN.1 Object Identifier
  InvalidBase128ObjectIdentifier,
  /// Invalid ASN.1 Octetstring
  InvalidOctetstring,
  /// Invalid ASN.1 Set
  InvalidSet,
  /// Invalid ASN.1 time
  InvalidTime,
  /// Invalid ASN.1 Tag-Length-Value
  InvalidTlv,
}
