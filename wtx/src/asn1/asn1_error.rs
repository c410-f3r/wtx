/// ASN.1 Error
#[derive(Debug)]
pub enum Asn1Error {
  /// Invalid Any bytes
  InvalidAnyBytes,
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
  /// Invalid Object Identifier Base128 bytes
  InvalidOidBase128,
  /// Invalid Object Identifier bytes
  InvalidOidBytes,
  /// Invalid Octetstring
  InvalidOctetstring,
  /// Invalid Set
  InvalidSet,
  /// Invalid Tag-Length-Value
  InvalidTlv,
  /// A sequence of DER bytes can not represent `u32`
  InvalidU32Bytes,
  /// Invalid UTC Time
  InvalidUtcTime,
  /// ASN.1 data can not be greater than `u16::MAX`
  LargeData,
}
