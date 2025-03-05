create_enum! {
  /// Aggregation of MySQL e MariaDB capabilities
  #[derive(Debug, Clone, Copy)]
  pub(crate) enum Capability<u64> {
    Mysql = (1),
    FoundRows = (2),
    LongFlag = (4),
    ConnectWithDb = (8),
    NoSchema = (16),
    Compress = (32),
    Odbc = (64),
    LocalFiles = (128),
    IgnoreSpace = (256),
    Protocol41 = (512),
    Interactive = (1024),
    Ssl = (2048),
    Transactions = (16384),
    SecureConnection = (32768),
    MultiStatements = (65536), // 1 << 16
    MultiResults = (131072), // 1 << 17
    PsMultiResults = (262144), // 1 << 18
    PluginAuth = (524288), // 1 << 19
    ConnectAttrs = (1048576), // 1 << 20
    PluginAuthLenencData = (2097152), // 1 << 21
    CanHandleExpiredPasswords = (4194304), // 1 << 22
    SessionTrack = (8388608), // 1 << 23
    DeprecateEof = (16777216),  // 1 << 24
    OptionalResultsetMetadata = (33554432), // 1 << 25
    ZstdCompressionAlgorithm = (67108864), // 1 << 26
    QueryAttributes = (134217728), // 1 << 27
    SslVerifyServerCert = (1073741824), // 1 << 30
    RememberOptions = (2147483648), // 1 << 31
    MariaDbClientProgress = (4294967296), // 1 << 32
    MariaDbClientMulti = (8589934592),  // 1 << 33
    MariaDbClientStmtBulkOperations = (17179869184), // 1 << 34
    MariaDbClientExtendedTypeInfo = (34359738368), // 1 << 35
    MariaDbClientCacheMetadata = (68719476736), // 1 << 36
    MariaDbClientBulkUnitResults = (137438953472) // 1 << 37
  }
}
