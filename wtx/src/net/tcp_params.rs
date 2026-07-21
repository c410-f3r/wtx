/// Configuration parameters for establishing TCP connections.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TcpParams {
  pub(crate) listen: i32,
  pub(crate) reuse_address: Option<bool>,
  pub(crate) reuse_port: Option<bool>,
  pub(crate) tcp_nodelay: bool,
}

impl TcpParams {
  /// The maximum number of queued incoming connections (backlog).
  ///
  /// NO-OP if used in a client.
  #[inline]
  #[must_use]
  pub const fn listen(mut self, value: i32) -> Self {
    self.listen = value;
    self
  }

  /// Sets the `SO_REUSEADDR` socket option.
  #[inline]
  #[must_use]
  pub const fn reuse_address(mut self, value: Option<bool>) -> Self {
    self.reuse_address = value;
    self
  }

  /// Sets the `SO_REUSEPORT` socket option.
  #[inline]
  #[must_use]
  pub const fn reuse_port(mut self, value: Option<bool>) -> Self {
    self.reuse_port = value;
    self
  }

  /// Sets the `TCP_NODELAY` socket option.
  #[inline]
  #[must_use]
  pub const fn tcp_nodelay(mut self, value: bool) -> Self {
    self.tcp_nodelay = value;
    self
  }
}

impl Default for TcpParams {
  #[inline]
  fn default() -> Self {
    Self { listen: 4096, reuse_address: Some(true), reuse_port: Some(true), tcp_nodelay: true }
  }
}
