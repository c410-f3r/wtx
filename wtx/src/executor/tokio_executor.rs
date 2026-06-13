use crate::{
  executor::{Executor, ExecutorTy, Runtime, TcpListener, TcpStream, address, tcp_listener_std},
  misc::TcpParams,
};
use core::{
  net::SocketAddr,
  pin::{Pin, pin},
  task::{Context, Poll, ready},
};
use tokio::{
  net::tcp::{OwnedReadHalf, OwnedWriteHalf},
  runtime::{Builder, LocalOptions},
};

#[derive(Clone, Debug)]
pub struct TokioExecutor;

impl Executor for TokioExecutor {
  const TY: ExecutorTy = ExecutorTy::Tokio;

  type LocalRuntime = tokio::runtime::LocalRuntime;
  type SpawnFuture<T> = SpawnFuture<T>;
  type TcpListener = tokio::net::TcpListener;
  type TcpStream = tokio::net::TcpStream;

  #[inline]
  fn spawn<F>(&self, future: F) -> Self::SpawnFuture<F::Output>
  where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
  {
    SpawnFuture(tokio::spawn(future))
  }
}

impl TcpListener for tokio::net::TcpListener {
  type TcpStream = tokio::net::TcpStream;

  #[inline]
  async fn bind(addr: (&str, u16), tcp_params: TcpParams) -> crate::Result<Self> {
    let socket = tcp_listener_std(address(addr)?, tcp_params)?;
    Ok(tokio::net::TcpListener::from_std(std::net::TcpListener::from(socket))?)
  }

  async fn accept(&self) -> crate::Result<(Self::TcpStream, SocketAddr)> {
    Ok((*self).accept().await?)
  }
}

impl Runtime for tokio::runtime::LocalRuntime {
  fn optioned() -> crate::Result<Self> {
    Ok(Builder::new_current_thread().enable_all().build_local(LocalOptions::default())?)
  }

  fn block_on<F>(&self, future: F) -> F::Output
  where
    F: Future,
  {
    (*self).block_on(future)
  }
}

impl TcpStream for tokio::net::TcpStream {
  type Executor = TokioExecutor;
  type ReadHalf = OwnedReadHalf;
  type WriteHalf = OwnedWriteHalf;

  #[inline]
  async fn connect(addr: (&str, u16)) -> crate::Result<Self> {
    Ok(tokio::net::TcpStream::connect(addr).await?)
  }

  #[inline]
  fn into_split(self) -> crate::Result<(Self::ReadHalf, Self::WriteHalf)> {
    Ok(self.into_split())
  }

  #[inline]
  fn peer_addr(&self) -> crate::Result<SocketAddr> {
    Ok((*self).peer_addr()?)
  }
}

pub struct SpawnFuture<T>(tokio::task::JoinHandle<T>);

impl<T> Future for SpawnFuture<T> {
  type Output = crate::Result<T>;

  #[inline]
  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    Poll::Ready(Ok(ready!(pin!(&mut self.0).poll(cx))?))
  }
}
