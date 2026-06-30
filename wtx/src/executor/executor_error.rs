/// Executor Error
#[derive(Clone, Copy, Debug)]
pub enum ExecutorError {
  /// The std runtime does not implement `spawn_local`
  UnsupportedStdSpawnLocal,
}
