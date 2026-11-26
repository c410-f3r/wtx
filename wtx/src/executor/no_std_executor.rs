use crate::{
  collections::MaybeUninitSlice,
  executor::{
    Executor, ExecutorError, ExecutorTy, NoStdRuntime, Runtime as _, TcpListener, TcpStream,
  },
  net::{Stream, StreamCommon, StreamReader, StreamWriter, TcpParams, ToSocketAddrs},
};
use alloc::boxed::Box;
use core::{
  fmt::{Debug, Formatter},
  net::{Ipv4Addr, SocketAddr, SocketAddrV4},
  num::NonZeroUsize,
  pin::Pin,
  task::{Context, Poll},
};

/// Executor for `no_std` environments.
///
/// At the current time the implementation is quite inefficient.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoStdExecutor {}

impl Executor for NoStdExecutor {
  const TY: ExecutorTy = ExecutorTy::NoStd;

  type LocalRuntime = NoStdRuntime;
  type SpawnFuture<T> = NoStdSpawnFuture<T>;
  type SpawnLocalFuture<T> = NoStdSpawnLocalFuture<T>;
  type TcpListener = TcpListenerDummy;
  type TcpStream = TcpStreamDummy;

  #[inline]
  fn spawn<F>(&self, future: F) -> Self::SpawnFuture<F::Output>
  where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
  {
    NoStdSpawnFuture { future: Some(Box::pin(future)) }
  }

  #[inline]
  fn spawn_local<F>(&self, future: F, _: &Self::LocalRuntime) -> Self::SpawnLocalFuture<F::Output>
  where
    F: Future + 'static,
    F::Output: 'static,
  {
    NoStdSpawnLocalFuture { future: Some(Box::pin(future)) }
  }
}

/// Returned by [`NoStdExecutor::spawn`].
pub struct NoStdSpawnFuture<T> {
  future: Option<Pin<Box<dyn Future<Output = T> + Send>>>,
}

impl<T> Debug for NoStdSpawnFuture<T> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("NoStdSpawnFuture").finish()
  }
}

impl<T> Future for NoStdSpawnFuture<T> {
  type Output = crate::Result<T>;

  #[inline]
  fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
    let Self { future } = self.get_mut();
    let Some(elem) = future.as_mut() else {
      return Poll::Ready(Err(ExecutorError::ExpiredFuture.into()));
    };
    let output = NoStdRuntime::new().block_on(elem);
    *future = None;
    Poll::Ready(Ok(output))
  }
}

/// Returned by [`NoStdExecutor::spawn_local`].
pub struct NoStdSpawnLocalFuture<T> {
  future: Option<Pin<Box<dyn Future<Output = T>>>>,
}

impl<T> Debug for NoStdSpawnLocalFuture<T> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("NoStdSpawnLocalFuture").finish()
  }
}

impl<T> Future for NoStdSpawnLocalFuture<T> {
  type Output = crate::Result<T>;

  #[inline]
  fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
    let Self { future } = self.get_mut();
    let Some(elem) = future.as_mut() else {
      return Poll::Ready(Err(ExecutorError::ExpiredFuture.into()));
    };
    let output = NoStdRuntime::new().block_on(elem);
    *future = None;
    Poll::Ready(Ok(output))
  }
}

/// A TCP listener that does not perform any real network operation.
#[derive(Clone, Copy, Debug, Default)]
pub struct TcpListenerDummy {}

impl TcpListenerDummy {
  /// New instance
  #[inline]
  pub const fn new() -> Self {
    Self {}
  }
}

impl TcpListener for TcpListenerDummy {
  type TcpStream = TcpStreamDummy;

  #[inline]
  async fn bind<A>(_: A, _: TcpParams) -> crate::Result<Self>
  where
    A: ToSocketAddrs,
  {
    Ok(Self::new())
  }

  #[inline]
  async fn accept(&self, _: TcpParams) -> crate::Result<(Self::TcpStream, SocketAddr)> {
    Ok((TcpStreamDummy::new(), addr_dummy()))
  }
}

/// A TCP stream that does not perform any real network operation.
#[derive(Clone, Copy, Debug, Default)]
pub struct TcpStreamDummy {}

impl TcpStreamDummy {
  /// New instance
  #[inline]
  pub const fn new() -> Self {
    Self {}
  }
}

impl Stream for TcpStreamDummy {
  type BridgeOwned = ();
  type ReadHalfOwned = ();
  type WriteHalfOwned = ();

  #[inline]
  fn into_split(
    self,
  ) -> crate::Result<(Self::BridgeOwned, Self::ReadHalfOwned, Self::WriteHalfOwned)> {
    Ok(((), (), ()))
  }
}

impl StreamCommon for TcpStreamDummy {}

impl StreamReader for TcpStreamDummy {
  #[inline]
  async fn read(&mut self, _: MaybeUninitSlice<'_, u8>) -> crate::Result<Option<NonZeroUsize>> {
    Ok(None)
  }
}

impl StreamWriter for TcpStreamDummy {
  #[inline]
  async fn write_all(&mut self, _: &[u8]) -> crate::Result<()> {
    Ok(())
  }

  #[inline]
  async fn write_all_vectored(&mut self, _: &[&[u8]]) -> crate::Result<()> {
    Ok(())
  }
}

impl TcpStream for TcpStreamDummy {
  type Executor = NoStdExecutor;

  #[inline]
  async fn connect<A>(_: A, _: TcpParams) -> crate::Result<Self>
  where
    A: ToSocketAddrs,
  {
    Ok(Self::new())
  }

  #[inline]
  fn peer_addr(&self) -> crate::Result<SocketAddr> {
    Ok(addr_dummy())
  }
}

#[inline]
const fn addr_dummy() -> SocketAddr {
  SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))
}

#[cfg(test)]
mod tests {
  use crate::executor::{Executor, ExecutorError, NoStdExecutor, NoStdRuntime, Runtime};

  #[test]
  fn spawn() {
    let executor = NoStdExecutor::default();
    let runtime = NoStdRuntime::new();
    let mut future = executor.spawn(async { 7 });
    assert_eq!(runtime.block_on(&mut future).unwrap(), 7);
    assert!(matches!(
      runtime.block_on(&mut future),
      Err(crate::Error::Executor(ExecutorError::ExpiredFuture))
    ));
  }
}
