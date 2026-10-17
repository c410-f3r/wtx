create_enum! {
  /// Database
  #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
  pub enum DatabaseTy<u8> {
    /// Mock
    Mock = (0, "mock"),
    /// PostgreSQL
    Postgres = (1, "postgres"),
    /// Dummy
    Unit = (2, "unit")
  }
}
