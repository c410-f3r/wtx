/// ASN.1 Error
#[derive(Debug)]
pub enum Asn1Error {
  /// Invalid Object Identifier
  InvalidBase128ObjectIdentifier,
  /// Invalid Bit String
  InvalidBitString,
  /// Invalid Boolean
  InvalidBoolean,
  /// Invalid Generalized Time
  InvalidGeneralizedTime,
  /// Invalid Generic Sequence
  InvalidGenericSequence(u8, u8),
  /// Invalid Integer
  InvalidInteger,
  /// Invalid length
  InvalidLen,
  /// Invalid Octetstring
  InvalidOctetstring,
  /// A sequence of DER bytes can not represent a serial number.
  InvalidSerialNumberBytes,
  /// Invalid Set
  InvalidSet,
  /// Invalid Tag-Length-Value
  InvalidTlv,
  /// A sequence of DER bytes can not represent `u32`
  InvalidU32Bytes,
  /// Invalid UTC Time
  InvalidUtcTime,
}
