create_enum! {
  /// Aggregation of MySQL e MariaDB capabilities
  #[derive(Debug, Clone, Copy)]
  pub(crate) enum Capability<u64> {
    /// MySQL compatibility.
    Mysql = (1),
    /// Return found rows instead of affected rows in EOF_Packet.
    FoundRows = (2),
    /// Retrieve all column flags.
    LongFlag = (4),
    /// Specify database (schema) on connect.
    ConnectWithDb = (8),
    /// Disallow database.table.column notation.
    NoSchema = (16),
    /// Support compression protocol.
    Compress = (32),
    /// Handle ODBC-specific behavior.
    Odbc = (64),
    /// Support LOAD DATA LOCAL.
    LocalFiles = (128),
    /// Ignore spaces before '(' in queries.
    IgnoreSpace = (256),
    /// Use MySQL 4.1+ protocol.
    Protocol41 = (512),
    /// Interactive client mode.
    Interactive = (1024),
    /// Use SSL encryption.
    Ssl = (2048),
    /// Support transactions.
    Transactions = (16384),
    /// Use 4.1+ authentication.
    SecureConnection = (32768),
    /// Support multi-statement queries.
    MultiStatements = (165536),
    /// Support multi-result sets for COM_QUERY.
    MultiResults = (131072),
    /// Support multi-result sets for COM_STMT_PREPARE.
    PsMultiResults = (262144),
    /// Support plugin-based authentication.
    PluginAuth = (524288),
    /// Support connection attributes.
    ConnectAttrs = (1048576),
    /// Allow authentication responses > 255 bytes.
    PluginAuthLenencData = (2097152),
    /// Don't close connections for expired passwords.
    CanHandleExpiredPasswords = (4194304),
    /// Handle server state change info.
    SessionTrack = (8388608),
    /// Use OK_Packet instead of EOF_Packet.
    DeprecateEof = (16777216),
    /// Handle optional metadata in result sets.
    OptionalResultsetMetadata = (33554432),
    /// Support ZSTD compression.
    ZstdCompressionAlgorithm = (67108864),
    /// Verify server certificate.
    SslVerifyServerCert = (1073741824),
    /// Retain options after failed connects.
    RememberOptions = (2147483648),
    /// Support progress indicator (10.2+).
    MariaDbClientProgress = (4294967296),
    /// Support COM_MULTI protocol.
    MariaDbClientMulti = (8589934592),
    /// Support bulk inserts.
    MariaDbClientStmtBulkOperations = (17179869184),
    /// Add extended type metadata.
    MariaDbClientExtendedTypeInfo = (34359738368),
    /// Allow skipping metadata.
    MariaDbClientCacheMetadata = (68719476736),
    /// Bulk commands return result sets with affected rows and auto-increment values.
    MariaDbClientBulkUnitResults = (137438953472)
  }
}
