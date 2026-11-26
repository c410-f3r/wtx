/// Executor Error
#[derive(Clone, Copy, Debug)]
pub enum ExecutorError {
  /// Future should complete before a certain duration but didn't
  ExpiredFuture,
  /// Could not resolve to any address
  InvalidResolvedAddress,
  /// The std runtime does not implement `spawn_local`
  UnsupportedStdSpawnLocal,
}
