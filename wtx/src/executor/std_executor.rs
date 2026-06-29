use crate::{
  executor::{
    Executor, ExecutorTy, SpawnThreadedFut, StdRuntime, TcpListener, TcpStream, tcp_listener_std,
  },
  misc::TcpParams,
};
use core::{
  mem,
  net::SocketAddr,
  pin::Pin,
  task::{Context, Poll, ready},
};
use std::net::ToSocketAddrs as _;

/// Uses the structures originated from the standard library.
#[derive(Clone, Debug, Default)]
pub struct StdExecutor(StdRuntime);

impl Executor for StdExecutor {
  const TY: ExecutorTy = ExecutorTy::Std;

  type LocalRuntime = StdRuntime;
  type SpawnFuture<T> = StdSpawnFutureFut<T>;
  type TcpListener = std::net::TcpListener;
  type TcpStream = std::net::TcpStream;

  #[inline]
  async fn lookup_host(host: (&str, u16)) -> crate::Result<impl Iterator<Item = SocketAddr>> {
    Ok(host.to_socket_addrs()?)
  }

  #[inline]
  fn spawn<F>(&self, future: F) -> Self::SpawnFuture<F::Output>
  where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
  {
    StdSpawnFutureFut(self.0.spawn_threaded(future))
  }
}

impl TcpListener for std::net::TcpListener {
  type TcpStream = std::net::TcpStream;

  #[inline]
  async fn bind(addr: (&str, u16), tcp_params: TcpParams) -> crate::Result<Self> {
    tcp_listener_std::<StdExecutor>(addr, tcp_params).await
  }

  #[inline]
  async fn accept(&self, tcp_params: TcpParams) -> crate::Result<(Self::TcpStream, SocketAddr)> {
    let rslt = (*self).accept()?;
    rslt.0.set_nodelay(tcp_params.tcp_nodelay)?;
    Ok(rslt)
  }
}

impl TcpStream for std::net::TcpStream {
  type Executor = StdExecutor;

  #[inline]
  async fn connect(addr: (&str, u16), tcp_params: TcpParams) -> crate::Result<Self> {
    let stream = std::net::TcpStream::connect(addr)?;
    stream.set_nodelay(tcp_params.tcp_nodelay)?;
    Ok(stream)
  }

  #[inline]
  fn peer_addr(&self) -> crate::Result<SocketAddr> {
    Ok((*self).peer_addr()?)
  }
}

/// Returned by [`StdExecutor::spawn`].
#[derive(Debug)]
pub struct StdSpawnFutureFut<T>(crate::Result<SpawnThreadedFut<T>>);

impl<T> Future for StdSpawnFutureFut<T> {
  type Output = crate::Result<T>;

  #[inline]
  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let fut = match self.0.as_mut() {
      Ok(el) => el,
      // This branch can only occur once, therefore, multiple awakes will always face `Ok`.
      //
      // The replaced future can be anything, it is just a placeholder for the swap operation.
      Err(err) => return Poll::Ready(Err(mem::replace(err, crate::Error::ExpiredFuture))),
    };
    Poll::Ready(Ok(ready!(Pin::new(fut).poll(cx))))
  }
}
