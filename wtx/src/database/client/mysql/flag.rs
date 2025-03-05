create_enum! {
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub(crate) enum Flag<u16> {
    NotNull = (1),
    PrimaryKey = (2),
    UniqueKey = (4),
    MultipleKey = (8),
    Blob = (16),
    Unsigned = (32),
    Zerofill = (64),
    Binary = (128),
    Enum = (256),
    AutoIncrement = (512),
    Timestamp = (1024),
    Set = (2048),
    NoDefaultValue = (4096),
    OnUpdateNow = (8192),
    Num = (32768),
  }
}
