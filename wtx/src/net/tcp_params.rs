/// Configuration parameters for establishing TCP connections.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TcpParams {
  listen: i32,
  recv_buffer_size: Option<u32>,
  reuse_address: Option<bool>,
  reuse_port: Option<bool>,
  send_buffer_size: Option<u32>,
  tcp_nodelay: bool,
}

impl TcpParams {
  /// Returns the maximum number of queued incoming connections.
  #[inline]
  pub const fn listen(&self) -> i32 {
    self.listen
  }

  /// Returns the `SO_RCVBUF` socket option.
  #[inline]
  pub const fn recv_buffer_size(&self) -> Option<u32> {
    self.recv_buffer_size
  }

  /// Returns the `SO_REUSEADDR` socket option.
  #[inline]
  pub const fn reuse_address(&self) -> Option<bool> {
    self.reuse_address
  }

  /// Returns the `SO_REUSEPORT` socket option.
  #[inline]
  pub const fn reuse_port(&self) -> Option<bool> {
    self.reuse_port
  }

  /// Returns the `SO_SNDBUF` socket option.
  #[inline]
  pub const fn send_buffer_size(&self) -> Option<u32> {
    self.send_buffer_size
  }

  /// Sets the maximum number of queued incoming connections (backlog).
  ///
  /// NO-OP if used in a client.
  #[inline]
  #[must_use]
  pub const fn set_listen(mut self, value: i32) -> Self {
    self.listen = value;
    self
  }

  /// Sets the `SO_RCVBUF` socket option.
  #[inline]
  #[must_use]
  pub const fn set_recv_buffer_size(mut self, value: Option<u32>) -> Self {
    self.recv_buffer_size = value;
    self
  }

  /// Sets the `SO_REUSEADDR` socket option.
  #[inline]
  #[must_use]
  pub const fn set_reuse_address(mut self, value: Option<bool>) -> Self {
    self.reuse_address = value;
    self
  }

  /// Sets the `SO_REUSEPORT` socket option.
  #[inline]
  #[must_use]
  pub const fn set_reuse_port(mut self, value: Option<bool>) -> Self {
    self.reuse_port = value;
    self
  }

  /// Sets the `SO_SNDBUF` socket option.
  #[inline]
  #[must_use]
  pub const fn set_send_buffer_size(mut self, value: Option<u32>) -> Self {
    self.send_buffer_size = value;
    self
  }

  /// Sets the `TCP_NODELAY` socket option.
  #[inline]
  #[must_use]
  pub const fn set_tcp_nodelay(mut self, value: bool) -> Self {
    self.tcp_nodelay = value;
    self
  }

  /// Returns the `TCP_NODELAY` socket option.
  #[inline]
  pub const fn tcp_nodelay(&self) -> bool {
    self.tcp_nodelay
  }
}

impl Default for TcpParams {
  #[inline]
  fn default() -> Self {
    Self {
      listen: 4096,
      recv_buffer_size: None,
      reuse_address: Some(true),
      reuse_port: Some(true),
      send_buffer_size: None,
      tcp_nodelay: true,
    }
  }
}
