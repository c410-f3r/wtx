use crate::{
  executor::{
    Executor, ExecutorTy, SpawnThreadedFut, StdRuntime, TcpListener, TcpStream, address,
    tcp_listener_std,
  },
  misc::TcpParams,
};
use core::{
  mem,
  net::SocketAddr,
  pin::{Pin, pin},
  task::{Context, Poll, ready},
};

#[derive(Clone, Default)]
pub struct StdExecutor(StdRuntime);

impl Executor for StdExecutor {
  const TY: ExecutorTy = ExecutorTy::Std;

  type LocalRuntime = StdRuntime;
  type SpawnFuture<T> = SpawnFuture<T>;
  type TcpListener = std::net::TcpListener;
  type TcpStream = std::net::TcpStream;

  fn spawn<F>(&self, future: F) -> Self::SpawnFuture<F::Output>
  where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
  {
    SpawnFuture(self.0.spawn_threaded(future))
  }
}

impl TcpListener for std::net::TcpListener {
  type TcpStream = std::net::TcpStream;

  #[inline]
  async fn bind(addr: (&str, u16), tcp_params: TcpParams) -> crate::Result<Self> {
    let socket = tcp_listener_std(address(addr)?, tcp_params)?;
    Ok(std::net::TcpListener::from(socket))
  }

  async fn accept(&self) -> crate::Result<(Self::TcpStream, SocketAddr)> {
    Ok((*self).accept()?)
  }
}

impl TcpStream for std::net::TcpStream {
  type Executor = StdExecutor;
  type ReadHalf = std::net::TcpStream;
  type WriteHalf = std::net::TcpStream;

  #[inline]
  async fn connect(addr: (&str, u16)) -> crate::Result<Self> {
    Ok(std::net::TcpStream::connect(addr)?)
  }

  #[inline]
  fn into_split(self) -> crate::Result<(Self::ReadHalf, Self::WriteHalf)> {
    Ok((self.try_clone()?, self))
  }

  #[inline]
  fn peer_addr(&self) -> crate::Result<SocketAddr> {
    Ok((*self).peer_addr()?)
  }
}

pub struct SpawnFuture<T>(crate::Result<SpawnThreadedFut<T>>);

impl<T> Future for SpawnFuture<T> {
  type Output = crate::Result<T>;

  #[inline]
  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let fut = match self.0.as_mut() {
      Ok(el) => el,
      // This branch can only occur once, therefore, multiple awakes will always face `Ok`.
      Err(err) => return Poll::Ready(Err(mem::replace(err, crate::Error::ClosedConnection))),
    };
    Poll::Ready(Ok(ready!(pin!(fut).poll(cx))))
  }
}
