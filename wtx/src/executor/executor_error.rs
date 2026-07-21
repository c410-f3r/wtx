/// Executor Error
#[derive(Clone, Copy, Debug)]
pub enum ExecutorError {
  /// Could not resolve to any address
  InvalidResolvedAddress,
  /// The std runtime does not implement `spawn_local`
  UnsupportedStdSpawnLocal,
}
