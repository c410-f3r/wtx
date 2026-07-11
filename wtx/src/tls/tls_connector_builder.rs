use crate::{
  executor::{Executor, TcpStream as _},
  misc::{Lease, SingleTypeStorage, TcpParams, Uri},
  tls::{TlsConfig, TlsConnector},
};

/// Builds [`TlsConnector`].
#[derive(Debug)]
pub struct TlsConnectorBuilder<EX, U> {
  _executor: EX,
  tcp_params: TcpParams,
  uri: U,
}

#[cfg(feature = "std")]
impl<STR, U> TlsConnectorBuilder<crate::executor::StdExecutor, U>
where
  STR: Lease<str>,
  U: Lease<Uri<STR>> + SingleTypeStorage<Item = STR>,
{
  /// [`Self::new`] that uses the elements provided by the standard library
  #[inline]
  pub fn std(uri: U) -> Self {
    Self::new(crate::executor::StdExecutor::default(), uri)
  }
}

#[cfg(feature = "tokio")]
impl<STR, U> TlsConnectorBuilder<crate::executor::TokioExecutor, U>
where
  STR: Lease<str>,
  U: Lease<Uri<STR>> + SingleTypeStorage<Item = STR>,
{
  /// [`Self::new`] that uses the elements provided by the tokio project
  #[inline]
  pub fn tokio(uri: U) -> Self {
    Self::new(crate::executor::TokioExecutor::default(), uri)
  }
}

impl<EX, STR, U> TlsConnectorBuilder<EX, U>
where
  EX: Executor,
  STR: Lease<str>,
  U: Lease<Uri<STR>> + SingleTypeStorage<Item = STR>,
{
  /// New instance with the minimum amount of required parameters.
  #[inline]
  pub fn new(executor: EX, uri: U) -> Self {
    Self { _executor: executor, tcp_params: TcpParams::default(), uri }
  }

  /// Transforms itself into [`TlsConnector`] according to the internal parameters.
  #[inline]
  pub async fn build<RNG, TC, TM>(
    self,
    config: TC,
    rng: RNG,
  ) -> crate::Result<TlsConnector<RNG, EX::TcpStream, TC, U>>
  where
    TC: Lease<TlsConfig<TM>> + SingleTypeStorage<Item = TM>,
  {
    let addr = self.uri.lease().hostname_with_implied_port();
    let stream = EX::TcpStream::connect(addr, self.tcp_params).await?;
    Ok(TlsConnector::new(config, rng, stream, self.uri))
  }

  /// Owned version of [`Self::tcp_params`].
  #[inline]
  #[must_use]
  pub const fn set_tcp_params(mut self, value: TcpParams) -> Self {
    self.tcp_params = value;
    self
  }

  /// See [`TcpParams`].
  #[inline]
  pub const fn tcp_params(&self) -> &TcpParams {
    &self.tcp_params
  }

  /// Mutable version of [`Self::tcp_params`].
  #[inline]
  pub const fn tcp_params_mut(&mut self) -> &mut TcpParams {
    &mut self.tcp_params
  }
}
