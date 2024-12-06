#[derive(Debug, Copy, Clone)]
pub(crate) enum State {
  FullyShutdown,
  ReadShutdown,
  Stream,
  WriteShutdown,
}
