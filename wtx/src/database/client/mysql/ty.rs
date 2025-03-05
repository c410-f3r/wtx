create_enum! {
  /// MySQL type
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum Ty<u8> {
    /// Decimal
    Decimal = (0),
    /// Tiny
    Tiny = (1),
    /// Short
    Short = (2),
    /// Long
    Long = (3),
    /// Float
    Float = (4),
    /// Double
    Double = (5),
    /// Null
    Null = (6),
    /// Timestamp
    Timestamp = (7),
    /// LongLong
    LongLong = (8),
    /// Int24
    Int24 = (9),
    /// Date
    Date = (10),
    /// Time
    Time = (11),
    /// Datetime
    Datetime = (12),
    /// Year
    Year = (13),
    /// VarChar
    VarChar = (15),
    /// Bit
    Bit = (16),
    /// Json
    Json = (245),
    /// NewDecimal
    NewDecimal = (246),
    /// Enum
    Enum = (247),
    /// Set
    Set = (248),
    /// TinyBlob
    TinyBlob = (249),
    /// MediumBlob
    MediumBlob = (250),
    /// LongBlog
    LongBlob = (251),
    /// Blob
    Blob = (252),
    /// VarString
    VarString = (253),
    /// String
    String = (254),
    /// Geometry
    Geometry = (255),
  }
}
