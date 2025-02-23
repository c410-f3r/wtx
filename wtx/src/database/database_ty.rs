create_enum! {
  /// Database
  #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
  pub enum DatabaseTy<u8> {
    /// MySql
    Mysql = (1, "mysql"),
    /// PostgreSQL
    Postgres = (2, "postgres"),
    /// Unit (Dummy used for testing)
    Unit = (5, "unit")
  }
}
