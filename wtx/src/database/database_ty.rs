create_enum! {
  /// Database
  #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
  pub enum DatabaseTy<u8> {
    /// PostgreSQL
    Postgres = (2, "postgres"),
    /// Unit (Dummy used for testing)
    Unit = (5, "unit")
  }
}
