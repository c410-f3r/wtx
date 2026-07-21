use crate::{
  executor::{
    Executor, ExecutorError, ExecutorTy, SpawnFuture, StdRuntime, TcpListener, TcpStream,
    resolve_addrs, tcp_listener_std,
  },
  net::{TcpParams, ToSocketAddrs},
};
use core::{
  marker::PhantomData,
  mem,
  net::SocketAddr,
  pin::Pin,
  task::{Context, Poll, ready},
};

/// Uses the structures originated from the standard library.
#[derive(Clone, Debug, Default)]
pub struct StdExecutor(StdRuntime);

impl Executor for StdExecutor {
  const TY: ExecutorTy = ExecutorTy::Std;

  type LocalRuntime = StdRuntime;
  type SpawnFuture<T> = StdSpawnFuture<T>;
  type SpawnLocalFuture<T> = StdSpawnLocalFuture<T>;
  type TcpListener = std::net::TcpListener;
  type TcpStream = std::net::TcpStream;

  #[inline]
  fn spawn<F>(&self, future: F) -> Self::SpawnFuture<F::Output>
  where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
  {
    StdSpawnFuture(self.0.spawn(future))
  }

  #[inline]
  fn spawn_local<F>(&self, _future: F) -> Self::SpawnLocalFuture<F::Output>
  where
    F: Future + 'static,
    F::Output: 'static,
  {
    StdSpawnLocalFuture(PhantomData)
  }
}

impl TcpListener for std::net::TcpListener {
  type TcpStream = std::net::TcpStream;

  #[inline]
  async fn bind<A>(addr: A, tcp_params: TcpParams) -> crate::Result<Self>
  where
    A: ToSocketAddrs,
  {
    tcp_listener_std(addr, &StdExecutor::default(), tcp_params).await
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
  async fn connect<A>(addr: A, tcp_params: TcpParams) -> crate::Result<Self>
  where
    A: ToSocketAddrs,
  {
    let stream = resolve_addrs(addr, &StdExecutor::default(), async |socket_addr| {
      Ok(std::net::TcpStream::connect(socket_addr)?)
    })
    .await?;
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
pub struct StdSpawnFuture<T>(crate::Result<SpawnFuture<T>>);

impl<T> Future for StdSpawnFuture<T> {
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

/// Returned by [`StdExecutor::spawn_local`].
#[derive(Debug)]
pub struct StdSpawnLocalFuture<T>(PhantomData<T>);

impl<T> Future for StdSpawnLocalFuture<T> {
  type Output = crate::Result<T>;

  #[inline]
  fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
    Poll::Ready(Err(ExecutorError::UnsupportedStdSpawnLocal.into()))
  }
}
