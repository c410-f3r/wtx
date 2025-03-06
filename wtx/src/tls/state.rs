#[derive(Debug, Copy, Clone)]
pub(crate) enum State {
  FullyShutdown,
  ReadShutdown,
  Stream,
  WriteShutdown,
}

impl State {
  #[inline]
  pub(crate) fn is_readable(&self) -> bool {
    !matches!(*self, Self::ReadShutdown | Self::FullyShutdown)
  }

  #[inline]
  pub(crate) fn writeable(&self) -> bool {
    !matches!(*self, Self::WriteShutdown | Self::FullyShutdown)
  }
}
