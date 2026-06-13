#[derive(Clone, Copy, Debug)]
pub struct TcpParams {
  pub(crate) listen: Option<i32>,
  pub(crate) reuse_address: Option<bool>,
  pub(crate) reuse_port: Option<bool>,
  pub(crate) tcp_nodelay: bool,
}

impl TcpParams {
  #[inline]
  pub const fn listen(mut self, value: Option<i32>) -> Self {
    self.listen = value;
    self
  }

  #[inline]
  pub const fn reuse_address(mut self, value: Option<bool>) -> Self {
    self.reuse_address = value;
    self
  }

  #[inline]
  pub const fn reuse_port(mut self, value: Option<bool>) -> Self {
    self.reuse_port = value;
    self
  }

  #[inline]
  pub const fn tcp_nodelay(mut self, value: bool) -> Self {
    self.tcp_nodelay = value;
    self
  }
}

impl Default for TcpParams {
  fn default() -> Self {
    Self {
      listen: Some(4096),
      reuse_address: Some(true),
      reuse_port: Some(true),
      tcp_nodelay: true,
    }
  }
}
