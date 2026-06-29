use crate::{
  executor::{Executor, ExecutorTy, Runtime, TcpListener, TcpStream, tcp_listener_std},
  misc::TcpParams,
};
use core::{
  net::SocketAddr,
  pin::Pin,
  task::{Context, Poll, ready},
};
use tokio::runtime::{Builder, LocalOptions};

/// Uses the structures originated from the `tokio` project.
#[derive(Clone, Debug, Default)]
pub struct TokioExecutor {}

impl Executor for TokioExecutor {
  const TY: ExecutorTy = ExecutorTy::Tokio;

  type LocalRuntime = tokio::runtime::LocalRuntime;
  type SpawnFuture<T> = TokioSpawnFutureFut<T>;
  type TcpListener = tokio::net::TcpListener;
  type TcpStream = tokio::net::TcpStream;

  #[inline]
  async fn lookup_host(host: (&str, u16)) -> crate::Result<impl Iterator<Item = SocketAddr>> {
    Ok(tokio::net::lookup_host(host).await?)
  }

  #[inline]
  fn spawn<F>(&self, future: F) -> Self::SpawnFuture<F::Output>
  where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
  {
    TokioSpawnFutureFut(tokio::spawn(future))
  }
}

impl TcpListener for tokio::net::TcpListener {
  type TcpStream = tokio::net::TcpStream;

  #[inline]
  async fn bind(addr: (&str, u16), tcp_params: TcpParams) -> crate::Result<Self> {
    let tcp_listener = tcp_listener_std::<TokioExecutor>(addr, tcp_params).await?;
    tcp_listener.set_nonblocking(true)?;
    Ok(tokio::net::TcpListener::from_std(tcp_listener)?)
  }

  #[inline]
  async fn accept(&self, tcp_params: TcpParams) -> crate::Result<(Self::TcpStream, SocketAddr)> {
    let rslt = (*self).accept().await?;
    rslt.0.set_nodelay(tcp_params.tcp_nodelay)?;
    Ok(rslt)
  }
}

impl Runtime for tokio::runtime::LocalRuntime {
  #[inline]
  fn optioned() -> crate::Result<Self> {
    Ok(Builder::new_current_thread().enable_all().build_local(LocalOptions::default())?)
  }

  #[inline]
  fn block_on<F>(&self, future: F) -> F::Output
  where
    F: Future,
  {
    (*self).block_on(future)
  }
}

impl TcpStream for tokio::net::TcpStream {
  type Executor = TokioExecutor;

  #[inline]
  async fn connect(addr: (&str, u16), tcp_params: TcpParams) -> crate::Result<Self> {
    let tcp_stream = tokio::net::TcpStream::connect(addr).await?;
    tcp_stream.set_nodelay(tcp_params.tcp_nodelay)?;
    Ok(tcp_stream)
  }

  #[inline]
  fn peer_addr(&self) -> crate::Result<SocketAddr> {
    Ok((*self).peer_addr()?)
  }
}

/// Returned by [`TokioExecutor::spawn`].
#[derive(Debug)]
pub struct TokioSpawnFutureFut<T>(tokio::task::JoinHandle<T>);

impl<T> Future for TokioSpawnFutureFut<T> {
  type Output = crate::Result<T>;

  #[inline]
  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    Poll::Ready(Ok(ready!(Pin::new(&mut self.0).poll(cx))?))
  }
}
