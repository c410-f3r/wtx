create_enum! {
  /// Status
  #[derive(Clone, Copy, Debug)]
  pub(crate) enum Status<u16> {
    // A transaction is currently active
    ServerStatusInTrans = (1),
    // Autocommit mode is set
    ServerStatusAutocommit = (2),
    // Multi query - next query exists.
    ServerMoreResultsExists = (8),
    /// Indicates that an index was used but it was not optimal or effective for the
    /// query, leading to suboptimal performance.
    ServerQueryNoGoodIndexUsed = (16),
    /// Indicates that no index was used and a full table scan was performed.
    ServerQueryNoIndexUsed = (32),
    // when using COM_STMT_FETCH, indicate that current cursor still has result
    ServerStatusCursorExists = (64),
    // When using COM_STMT_FETCH, indicate that current cursor has finished to send results
    ServerStatusLastRowSent = (128),
    // Database has been dropped
    ServerStatusDbDropped = (256),
    // Current escape mode is "no backslash escape"
    ServerStatusNoBackslashEscapes = (512),
    // A DDL change did have an impact on an existing PREPARE (an automatic
    // re-prepare has been executed)
    ServerStatusMetadataChanged = (1024),
    // Last statement took more than the time value specified
    // in server variable long_query_time.
    ServerQueryWasSlow = (2048),
    // This result-set contain stored procedure output parameter.
    ServerPsOutParams = (4096),
    // Current transaction is a read-only transaction.
    ServerStatusInTransReadonly = (8192),
    // This status flag, when on, implies that one of the state information has changed
    // on the server because of the execution of the last statement.
    ServerSessionStateChanged = (16384),
  }
}
