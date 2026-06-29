/// Executor Error
#[derive(Clone, Copy, Debug)]
pub enum ExecutorError {
  /// For example, when calling `send_data_concurrent`.
  ClosedConnectionWhenSendingConcurrentData,
}