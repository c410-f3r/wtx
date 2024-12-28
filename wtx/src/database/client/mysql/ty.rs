/// Type
#[derive(Debug, PartialEq)]
pub enum Ty {
  /// Decimal
  Decimal = 0x00,
  /// Tiny
  Tiny = 0x01,
  /// Shoart
  Short = 0x02,
  /// Long
  Long = 0x03,
  /// Float
  Float = 0x04,
  /// Double
  Double = 0x05,
  /// Null
  Null = 0x06,
  /// Timestamp
  Timestamp = 0x07,
  /// LongLong
  LongLong = 0x08,
  /// Int24
  Int24 = 0x09,
  /// Date
  Date = 0x0a,
  /// Time
  Time = 0x0b,
  /// Datetime
  Datetime = 0x0c,
  /// Yead
  Year = 0x0d,
  /// VarChat
  VarChar = 0x0f,
  /// Bit
  Bit = 0x10,
  /// Json
  Json = 0xf5,
  /// NewDecimal
  NewDecimal = 0xf6,
  /// Enum
  Enum = 0xf7,
  /// Set
  Set = 0xf8,
  /// TinyBlob
  TinyBlob = 0xf9,
  /// MediumBlob
  MediumBlob = 0xfa,
  /// LongBlob
  LongBlob = 0xfb,
  /// Blob
  Blob = 0xfc,
  /// VarString
  VarString = 0xfd,
  /// String
  String = 0xfe,
  /// Geometry
  Geometry = 0xff,
}
