create_enum! {
  /// Database
  #[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
  pub enum DatabaseTy<u8> {
    /// MS-SQL
    Mssql = (0, "mssql"),
    /// MySql
    Mysql = (1, "mysql"),
    /// PostgreSQL
    #[default]
    Postgres = (2, "postgres"),
    /// Redis
    Redis = (3, "redis"),
    /// SQLite
    Sqlite = (4, "sqlite"),
    /// Unit (Dummy used for testing)
    Unit = (5, "unit")
  }
}
