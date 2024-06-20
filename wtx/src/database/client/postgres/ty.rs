/// Type
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ty {
  /// Bool
  Bool,
  /// Bytea
  Bytea,
  /// Char
  Char,
  /// Name
  Name,
  /// Int8
  Int8,
  /// Int2
  Int2,
  /// Int2Vector
  Int2Vector,
  /// Int4
  Int4,
  /// Regproc
  Regproc,
  /// Text
  Text,
  /// Oid
  Oid,
  /// Tid
  Tid,
  /// Xid
  Xid,
  /// Cid
  Cid,
  /// OidVector
  OidVector,
  /// PgDdlCommand
  PgDdlCommand,
  /// Json
  Json,
  /// Xml
  Xml,
  /// XmlArray
  XmlArray,
  /// PgNodeTree
  PgNodeTree,
  /// JsonArray
  JsonArray,
  /// TableAmHandler
  TableAmHandler,
  /// Xid8Array
  Xid8Array,
  /// IndexAmHandler
  IndexAmHandler,
  /// Point
  Point,
  /// Lseg
  Lseg,
  /// Path
  Path,
  /// Box
  Box,
  /// Polygon
  Polygon,
  /// Line
  Line,
  /// LineArray
  LineArray,
  /// Cidr
  Cidr,
  /// CidrArray
  CidrArray,
  /// Float4
  Float4,
  /// Float8
  Float8,
  /// Unknown
  Unknown,
  /// Circle
  Circle,
  /// CircleArray
  CircleArray,
  /// Macaddr8
  Macaddr8,
  /// Macaddr8Array
  Macaddr8Array,
  /// Money
  Money,
  /// MoneyArray
  MoneyArray,
  /// Macaddr
  Macaddr,
  /// Inet
  Inet,
  /// BoolArray
  BoolArray,
  /// ByteaArray
  ByteaArray,
  /// CharArray
  CharArray,
  /// NameArray
  NameArray,
  /// Int2Array
  Int2Array,
  /// Int2VectorArray
  Int2VectorArray,
  /// Int4Array
  Int4Array,
  /// RegprocArray
  RegprocArray,
  /// TextArray
  TextArray,
  /// TidArray
  TidArray,
  /// XidArray
  XidArray,
  /// CidArray
  CidArray,
  /// OidVectorArray
  OidVectorArray,
  /// BpcharArray
  BpcharArray,
  /// VarcharArray
  VarcharArray,
  /// Int8Array
  Int8Array,
  /// PointArray
  PointArray,
  /// LsegArray
  LsegArray,
  /// PathArray
  PathArray,
  /// BoxArray
  BoxArray,
  /// Float4Array
  Float4Array,
  /// Float8Array
  Float8Array,
  /// PolygonArray
  PolygonArray,
  /// OidArray
  OidArray,
  /// Aclitem
  Aclitem,
  /// AclitemArray
  AclitemArray,
  /// MacaddrArray
  MacaddrArray,
  /// InetArray
  InetArray,
  /// Bpchar
  Bpchar,
  /// Varchar
  Varchar,
  /// Date
  Date,
  /// Time
  Time,
  /// Timestamp
  Timestamp,
  /// TimestampArray
  TimestampArray,
  /// DateArray
  DateArray,
  /// TimeArray
  TimeArray,
  /// Timestamptz
  Timestamptz,
  /// TimestamptzArray
  TimestamptzArray,
  /// Interval
  Interval,
  /// IntervalArray
  IntervalArray,
  /// NumericArray
  NumericArray,
  /// CstringArray
  CstringArray,
  /// Timetz
  Timetz,
  /// TimetzArray
  TimetzArray,
  /// Bit
  Bit,
  /// BitArray
  BitArray,
  /// Varbit
  Varbit,
  /// VarbitArray
  VarbitArray,
  /// Numeric
  Numeric,
  /// Refcursor
  Refcursor,
  /// RefcursorArray
  RefcursorArray,
  /// Regprocedure
  Regprocedure,
  /// Regoper
  Regoper,
  /// Regoperator
  Regoperator,
  /// Regclass
  Regclass,
  /// Regtype
  Regtype,
  /// RegprocedureArray
  RegprocedureArray,
  /// RegoperArray
  RegoperArray,
  /// RegoperatorArray
  RegoperatorArray,
  /// RegclassArray
  RegclassArray,
  /// RegtypeArray
  RegtypeArray,
  /// Record
  Record,
  /// Cstring
  Cstring,
  /// Any
  Any,
  /// Anyarray
  Anyarray,
  /// Void
  Void,
  /// Trigger
  Trigger,
  /// LanguageHandler
  LanguageHandler,
  /// Internal
  Internal,
  /// Anyelement
  Anyelement,
  /// RecordArray
  RecordArray,
  /// Anynonarray
  Anynonarray,
  /// TxidSnapshotArray
  TxidSnapshotArray,
  /// Uuid
  Uuid,
  /// UuidArray
  UuidArray,
  /// TxidSnapshot
  TxidSnapshot,
  /// FdwHandler
  FdwHandler,
  /// PgLsn
  PgLsn,
  /// PgLsnArray
  PgLsnArray,
  /// TsmHandler
  TsmHandler,
  /// PgNdistinct
  PgNdistinct,
  /// PgDependencies
  PgDependencies,
  /// Anyenum
  Anyenum,
  /// TsVector
  TsVector,
  /// Tsquery
  Tsquery,
  /// GtsVector
  GtsVector,
  /// TsVectorArray
  TsVectorArray,
  /// GtsVectorArray
  GtsVectorArray,
  /// TsqueryArray
  TsqueryArray,
  /// Regconfig
  Regconfig,
  /// RegconfigArray
  RegconfigArray,
  /// Regdictionary
  Regdictionary,
  /// RegdictionaryArray
  RegdictionaryArray,
  /// Jsonb
  Jsonb,
  /// JsonbArray
  JsonbArray,
  /// AnyRange
  AnyRange,
  /// EventTrigger
  EventTrigger,
  /// Int4Range
  Int4Range,
  /// Int4RangeArray
  Int4RangeArray,
  /// NumRange
  NumRange,
  /// NumRangeArray
  NumRangeArray,
  /// TsRange
  TsRange,
  /// TsRangeArray
  TsRangeArray,
  /// TstzRange
  TstzRange,
  /// TstzRangeArray
  TstzRangeArray,
  /// DateRange
  DateRange,
  /// DateRangeArray
  DateRangeArray,
  /// Int8Range
  Int8Range,
  /// Int8RangeArray
  Int8RangeArray,
  /// Jsonpath
  Jsonpath,
  /// JsonpathArray
  JsonpathArray,
  /// Regnamespace
  Regnamespace,
  /// RegnamespaceArray
  RegnamespaceArray,
  /// Regrole
  Regrole,
  /// RegroleArray
  RegroleArray,
  /// Regcollation
  Regcollation,
  /// RegcollationArray
  RegcollationArray,
  /// PgBrinBloomSummary
  PgBrinBloomSummary,
  /// PgBrinMinmaxMultiSummary
  PgBrinMinmaxMultiSummary,
  /// PgMcvList
  PgMcvList,
  /// PgSnapshot
  PgSnapshot,
  /// PgSnapshotArray
  PgSnapshotArray,
  /// Xid8
  Xid8,
  /// Anycompatible
  Anycompatible,
  /// Anycompatiblearray
  Anycompatiblearray,
  /// Anycompatiblenonarray
  Anycompatiblenonarray,
  /// AnycompatibleRange
  AnycompatibleRange,
  /// Custom
  Custom(u32),
}

impl From<&Ty> for u32 {
  #[allow(
    // False positive
    clippy::too_many_lines
  )]
  #[inline]
  fn from(from: &Ty) -> Self {
    match from {
      Ty::Bool => 16,
      Ty::Bytea => 17,
      Ty::Char => 18,
      Ty::Name => 19,
      Ty::Int8 => 20,
      Ty::Int2 => 21,
      Ty::Int2Vector => 22,
      Ty::Int4 => 23,
      Ty::Regproc => 24,
      Ty::Text => 25,
      Ty::Oid => 26,
      Ty::Tid => 27,
      Ty::Xid => 28,
      Ty::Cid => 29,
      Ty::OidVector => 30,
      Ty::PgDdlCommand => 32,
      Ty::Json => 114,
      Ty::Xml => 142,
      Ty::XmlArray => 143,
      Ty::PgNodeTree => 194,
      Ty::JsonArray => 199,
      Ty::TableAmHandler => 269,
      Ty::Xid8Array => 271,
      Ty::IndexAmHandler => 325,
      Ty::Point => 600,
      Ty::Lseg => 601,
      Ty::Path => 602,
      Ty::Box => 603,
      Ty::Polygon => 604,
      Ty::Line => 628,
      Ty::LineArray => 629,
      Ty::Cidr => 650,
      Ty::CidrArray => 651,
      Ty::Float4 => 700,
      Ty::Float8 => 701,
      Ty::Unknown => 705,
      Ty::Circle => 718,
      Ty::CircleArray => 719,
      Ty::Macaddr8 => 774,
      Ty::Macaddr8Array => 775,
      Ty::Money => 790,
      Ty::MoneyArray => 791,
      Ty::Macaddr => 829,
      Ty::Inet => 869,
      Ty::BoolArray => 1000,
      Ty::ByteaArray => 1001,
      Ty::CharArray => 1002,
      Ty::NameArray => 1003,
      Ty::Int2Array => 1005,
      Ty::Int2VectorArray => 1006,
      Ty::Int4Array => 1007,
      Ty::RegprocArray => 1008,
      Ty::TextArray => 1009,
      Ty::TidArray => 1010,
      Ty::XidArray => 1011,
      Ty::CidArray => 1012,
      Ty::OidVectorArray => 1013,
      Ty::BpcharArray => 1014,
      Ty::VarcharArray => 1015,
      Ty::Int8Array => 1016,
      Ty::PointArray => 1017,
      Ty::LsegArray => 1018,
      Ty::PathArray => 1019,
      Ty::BoxArray => 1020,
      Ty::Float4Array => 1021,
      Ty::Float8Array => 1022,
      Ty::PolygonArray => 1027,
      Ty::OidArray => 1028,
      Ty::Aclitem => 1033,
      Ty::AclitemArray => 1034,
      Ty::MacaddrArray => 1040,
      Ty::InetArray => 1041,
      Ty::Bpchar => 1042,
      Ty::Varchar => 1043,
      Ty::Date => 1082,
      Ty::Time => 1083,
      Ty::Timestamp => 1114,
      Ty::TimestampArray => 1115,
      Ty::DateArray => 1182,
      Ty::TimeArray => 1183,
      Ty::Timestamptz => 1184,
      Ty::TimestamptzArray => 1185,
      Ty::Interval => 1186,
      Ty::IntervalArray => 1187,
      Ty::NumericArray => 1231,
      Ty::CstringArray => 1263,
      Ty::Timetz => 1266,
      Ty::TimetzArray => 1270,
      Ty::Bit => 1560,
      Ty::BitArray => 1561,
      Ty::Varbit => 1562,
      Ty::VarbitArray => 1563,
      Ty::Numeric => 1700,
      Ty::Refcursor => 1790,
      Ty::RefcursorArray => 2201,
      Ty::Regprocedure => 2202,
      Ty::Regoper => 2203,
      Ty::Regoperator => 2204,
      Ty::Regclass => 2205,
      Ty::Regtype => 2206,
      Ty::RegprocedureArray => 2207,
      Ty::RegoperArray => 2208,
      Ty::RegoperatorArray => 2209,
      Ty::RegclassArray => 2210,
      Ty::RegtypeArray => 2211,
      Ty::Record => 2249,
      Ty::Cstring => 2275,
      Ty::Any => 2276,
      Ty::Anyarray => 2277,
      Ty::Void => 2278,
      Ty::Trigger => 2279,
      Ty::LanguageHandler => 2280,
      Ty::Internal => 2281,
      Ty::Anyelement => 2283,
      Ty::RecordArray => 2287,
      Ty::Anynonarray => 2776,
      Ty::TxidSnapshotArray => 2949,
      Ty::Uuid => 2950,
      Ty::UuidArray => 2951,
      Ty::TxidSnapshot => 2970,
      Ty::FdwHandler => 3115,
      Ty::PgLsn => 3220,
      Ty::PgLsnArray => 3221,
      Ty::TsmHandler => 3310,
      Ty::PgNdistinct => 3361,
      Ty::PgDependencies => 3402,
      Ty::Anyenum => 3500,
      Ty::TsVector => 3614,
      Ty::Tsquery => 3615,
      Ty::GtsVector => 3642,
      Ty::TsVectorArray => 3643,
      Ty::GtsVectorArray => 3644,
      Ty::TsqueryArray => 3645,
      Ty::Regconfig => 3734,
      Ty::RegconfigArray => 3735,
      Ty::Regdictionary => 3769,
      Ty::RegdictionaryArray => 3770,
      Ty::Jsonb => 3802,
      Ty::JsonbArray => 3807,
      Ty::AnyRange => 3831,
      Ty::EventTrigger => 3838,
      Ty::Int4Range => 3904,
      Ty::Int4RangeArray => 3905,
      Ty::NumRange => 3906,
      Ty::NumRangeArray => 3907,
      Ty::TsRange => 3908,
      Ty::TsRangeArray => 3909,
      Ty::TstzRange => 3910,
      Ty::TstzRangeArray => 3911,
      Ty::DateRange => 3912,
      Ty::DateRangeArray => 3913,
      Ty::Int8Range => 3926,
      Ty::Int8RangeArray => 3927,
      Ty::Jsonpath => 4072,
      Ty::JsonpathArray => 4073,
      Ty::Regnamespace => 4089,
      Ty::RegnamespaceArray => 4090,
      Ty::Regrole => 4096,
      Ty::RegroleArray => 4097,
      Ty::Regcollation => 4191,
      Ty::RegcollationArray => 4192,
      Ty::PgBrinBloomSummary => 4600,
      Ty::PgBrinMinmaxMultiSummary => 4601,
      Ty::PgMcvList => 5017,
      Ty::PgSnapshot => 5038,
      Ty::PgSnapshotArray => 5039,
      Ty::Xid8 => 5069,
      Ty::Anycompatible => 5077,
      Ty::Anycompatiblearray => 5078,
      Ty::Anycompatiblenonarray => 5079,
      Ty::AnycompatibleRange => 5080,
      Ty::Custom(elem) => *elem,
    }
  }
}

impl TryFrom<u32> for Ty {
  type Error = crate::Error;

  #[allow(
    // False positive
    clippy::too_many_lines
  )]
  #[inline]
  fn try_from(from: u32) -> Result<Self, Self::Error> {
    Ok(match from {
      16 => Self::Bool,
      17 => Self::Bytea,
      18 => Self::Char,
      19 => Self::Name,
      20 => Self::Int8,
      21 => Self::Int2,
      22 => Self::Int2Vector,
      23 => Self::Int4,
      24 => Self::Regproc,
      25 => Self::Text,
      26 => Self::Oid,
      27 => Self::Tid,
      28 => Self::Xid,
      29 => Self::Cid,
      30 => Self::OidVector,
      32 => Self::PgDdlCommand,
      114 => Self::Json,
      142 => Self::Xml,
      143 => Self::XmlArray,
      194 => Self::PgNodeTree,
      199 => Self::JsonArray,
      269 => Self::TableAmHandler,
      271 => Self::Xid8Array,
      325 => Self::IndexAmHandler,
      600 => Self::Point,
      601 => Self::Lseg,
      602 => Self::Path,
      603 => Self::Box,
      604 => Self::Polygon,
      628 => Self::Line,
      629 => Self::LineArray,
      650 => Self::Cidr,
      651 => Self::CidrArray,
      700 => Self::Float4,
      701 => Self::Float8,
      705 => Self::Unknown,
      718 => Self::Circle,
      719 => Self::CircleArray,
      774 => Self::Macaddr8,
      775 => Self::Macaddr8Array,
      790 => Self::Money,
      791 => Self::MoneyArray,
      829 => Self::Macaddr,
      869 => Self::Inet,
      1000 => Self::BoolArray,
      1001 => Self::ByteaArray,
      1002 => Self::CharArray,
      1003 => Self::NameArray,
      1005 => Self::Int2Array,
      1006 => Self::Int2VectorArray,
      1007 => Self::Int4Array,
      1008 => Self::RegprocArray,
      1009 => Self::TextArray,
      1010 => Self::TidArray,
      1011 => Self::XidArray,
      1012 => Self::CidArray,
      1013 => Self::OidVectorArray,
      1014 => Self::BpcharArray,
      1015 => Self::VarcharArray,
      1016 => Self::Int8Array,
      1017 => Self::PointArray,
      1018 => Self::LsegArray,
      1019 => Self::PathArray,
      1020 => Self::BoxArray,
      1021 => Self::Float4Array,
      1022 => Self::Float8Array,
      1027 => Self::PolygonArray,
      1028 => Self::OidArray,
      1033 => Self::Aclitem,
      1034 => Self::AclitemArray,
      1040 => Self::MacaddrArray,
      1041 => Self::InetArray,
      1042 => Self::Bpchar,
      1043 => Self::Varchar,
      1082 => Self::Date,
      1083 => Self::Time,
      1114 => Self::Timestamp,
      1115 => Self::TimestampArray,
      1182 => Self::DateArray,
      1183 => Self::TimeArray,
      1184 => Self::Timestamptz,
      1185 => Self::TimestamptzArray,
      1186 => Self::Interval,
      1187 => Self::IntervalArray,
      1231 => Self::NumericArray,
      1263 => Self::CstringArray,
      1266 => Self::Timetz,
      1270 => Self::TimetzArray,
      1560 => Self::Bit,
      1561 => Self::BitArray,
      1562 => Self::Varbit,
      1563 => Self::VarbitArray,
      1700 => Self::Numeric,
      1790 => Self::Refcursor,
      2201 => Self::RefcursorArray,
      2202 => Self::Regprocedure,
      2203 => Self::Regoper,
      2204 => Self::Regoperator,
      2205 => Self::Regclass,
      2206 => Self::Regtype,
      2207 => Self::RegprocedureArray,
      2208 => Self::RegoperArray,
      2209 => Self::RegoperatorArray,
      2210 => Self::RegclassArray,
      2211 => Self::RegtypeArray,
      2249 => Self::Record,
      2275 => Self::Cstring,
      2276 => Self::Any,
      2277 => Self::Anyarray,
      2278 => Self::Void,
      2279 => Self::Trigger,
      2280 => Self::LanguageHandler,
      2281 => Self::Internal,
      2283 => Self::Anyelement,
      2287 => Self::RecordArray,
      2776 => Self::Anynonarray,
      2949 => Self::TxidSnapshotArray,
      2950 => Self::Uuid,
      2951 => Self::UuidArray,
      2970 => Self::TxidSnapshot,
      3115 => Self::FdwHandler,
      3220 => Self::PgLsn,
      3221 => Self::PgLsnArray,
      3310 => Self::TsmHandler,
      3361 => Self::PgNdistinct,
      3402 => Self::PgDependencies,
      3500 => Self::Anyenum,
      3614 => Self::TsVector,
      3615 => Self::Tsquery,
      3642 => Self::GtsVector,
      3643 => Self::TsVectorArray,
      3644 => Self::GtsVectorArray,
      3645 => Self::TsqueryArray,
      3734 => Self::Regconfig,
      3735 => Self::RegconfigArray,
      3769 => Self::Regdictionary,
      3770 => Self::RegdictionaryArray,
      3802 => Self::Jsonb,
      3807 => Self::JsonbArray,
      3831 => Self::AnyRange,
      3838 => Self::EventTrigger,
      3904 => Self::Int4Range,
      3905 => Self::Int4RangeArray,
      3906 => Self::NumRange,
      3907 => Self::NumRangeArray,
      3908 => Self::TsRange,
      3909 => Self::TsRangeArray,
      3910 => Self::TstzRange,
      3911 => Self::TstzRangeArray,
      3912 => Self::DateRange,
      3913 => Self::DateRangeArray,
      3926 => Self::Int8Range,
      3927 => Self::Int8RangeArray,
      4072 => Self::Jsonpath,
      4073 => Self::JsonpathArray,
      4089 => Self::Regnamespace,
      4090 => Self::RegnamespaceArray,
      4096 => Self::Regrole,
      4097 => Self::RegroleArray,
      4191 => Self::Regcollation,
      4192 => Self::RegcollationArray,
      4600 => Self::PgBrinBloomSummary,
      4601 => Self::PgBrinMinmaxMultiSummary,
      5017 => Self::PgMcvList,
      5038 => Self::PgSnapshot,
      5039 => Self::PgSnapshotArray,
      5069 => Self::Xid8,
      5077 => Self::Anycompatible,
      5078 => Self::Anycompatiblearray,
      5079 => Self::Anycompatiblenonarray,
      5080 => Self::AnycompatibleRange,
      _ => return Err(crate::Error::MISC_UnexpectedUint { received: from }),
    })
  }
}
