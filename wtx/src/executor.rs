//! Groups elements related to async operations.

#![allow(clippy::disallowed_types, reason = "traits require the `Arc` from std")]

mod no_std_runtime;
#[cfg(feature = "std")]
mod std_executor;
#[cfg(feature = "std")]
mod std_runtime;
#[cfg(feature = "tokio")]
mod tokio_executor;

use crate::{
  misc::TcpParams,
  stream::{Stream, StreamReader, StreamWriter},
};
use core::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
pub use no_std_runtime::NoStdRuntime;
#[cfg(feature = "tokio")]
pub use tokio_executor::TokioExecutor;
#[cfg(feature = "std")]
pub use {
  std_executor::StdExecutor,
  std_runtime::{SpawnThreadedFut, StdRuntime},
};

#[derive(Clone, Copy, Debug)]
pub enum ExecutorTy {
  Std,
  Tokio,
}

/// Generic executor
pub trait Executor {
  /// See [`ExecutorTy`].
  const TY: ExecutorTy;

  /// Local runtime
  type LocalRuntime: Runtime;
  /// Future of [`Self::spawn`].
  type SpawnFuture<T>: Future<Output = crate::Result<T>>;
  /// See [`TcpListener`].
  type TcpListener: TcpListener<TcpStream = Self::TcpStream>;
  /// See [`TcpStream`].
  type TcpStream: TcpStream<Executor = Self>;

  fn spawn<F>(&self, future: F) -> Self::SpawnFuture<F::Output>
  where
    F: Future + Send + 'static,
    F::Output: Send + 'static;
}

pub trait Runtime: Sized {
  fn optioned() -> crate::Result<Self>;

  fn block_on<F>(&self, future: F) -> F::Output
  where
    F: Future;
}

pub trait TcpListener: Sized {
  /// See [`TcpStream`].
  type TcpStream: TcpStream;

  fn bind(addr: (&str, u16), tcp_params: TcpParams) -> impl Future<Output = crate::Result<Self>>;

  fn accept(&self) -> impl Future<Output = crate::Result<(Self::TcpStream, SocketAddr)>>;
}

impl TcpListener for () {
  type TcpStream = ();

  #[inline]
  async fn bind(_: (&str, u16), _: TcpParams) -> crate::Result<Self> {
    Ok(())
  }

  #[inline]
  async fn accept(&self) -> crate::Result<(Self::TcpStream, SocketAddr)> {
    Ok(((), SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from_bits(0), 0))))
  }
}

pub trait TcpStream: Sized + Stream {
  // Used to type hint DB tests
  type Executor;
  type ReadHalf: StreamReader;
  type WriteHalf: StreamWriter;

  fn connect(addr: (&str, u16)) -> impl Future<Output = crate::Result<Self>>;

  fn into_split(self) -> crate::Result<(Self::ReadHalf, Self::WriteHalf)>;

  fn peer_addr(&self) -> crate::Result<SocketAddr>;
}

impl TcpStream for () {
  type Executor = ();
  type ReadHalf = ();
  type WriteHalf = ();

  #[inline]
  async fn connect(_: (&str, u16)) -> crate::Result<Self> {
    Ok(())
  }

  #[inline]
  fn into_split(self) -> crate::Result<(Self::ReadHalf, Self::WriteHalf)> {
    Ok(((), ()))
  }

  #[inline]
  fn peer_addr(&self) -> crate::Result<SocketAddr> {
    Ok(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from_bits(0), 0)))
  }
}

#[cfg(feature = "std")]
fn address(addr: (&str, u16)) -> crate::Result<SocketAddr> {
  use ::std::net::ToSocketAddrs;
  crate::misc::into_rslt(addr.to_socket_addrs()?.next())
}

#[cfg(feature = "std")]
fn tcp_listener_std(
  address: SocketAddr,
  tcp_params: TcpParams,
) -> Result<socket2::Socket, crate::Error> {
  let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::STREAM, None)?;
  if let Some(elem) = tcp_params.listen {
    socket.listen(elem)?;
  }
  socket.set_nonblocking(true)?;
  if let Some(elem) = tcp_params.reuse_address {
    socket.set_reuse_address(elem)?;
  }
  #[cfg(not(any(
    target_os = "cygwin",
    target_os = "illumos",
    target_os = "solaris",
    target_os = "wasi"
  )))]
  if let Some(elem) = tcp_params.reuse_port {
    socket.set_reuse_port(elem)?;
  }
  socket.set_tcp_nodelay(tcp_params.tcp_nodelay)?;
  socket.bind(&address.into())?;
  Ok(socket)
}
