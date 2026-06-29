//! Groups elements related to async operations.

#![allow(clippy::disallowed_types, reason = "traits require the `Arc` from std")]

mod no_std_runtime;
#[cfg(feature = "std")]
mod std_executor;
#[cfg(feature = "std")]
mod std_runtime;
#[cfg(feature = "tokio")]
mod tokio_executor;

use crate::{misc::TcpParams, stream::Stream};
use core::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
pub use no_std_runtime::NoStdRuntime;
#[cfg(feature = "tokio")]
pub use tokio_executor::{TokioExecutor, TokioSpawnFutureFut};
#[cfg(feature = "std")]
pub use {
  std_executor::{StdExecutor, StdSpawnFutureFut},
  std_runtime::{SpawnThreadedFut, StdRuntime},
};

/// Identifies the associated executor.
#[derive(Clone, Copy, Debug)]
pub enum ExecutorTy {
  /// See [`StdExecutor`].
  Std,
  /// See [`TokioExecutor`]
  Tokio,
}

/// Generic executor
pub trait Executor {
  /// See [`ExecutorTy`].
  const TY: ExecutorTy;

  /// Local runtime.
  type LocalRuntime: Runtime;
  /// Future of [`Self::spawn`].
  type SpawnFuture<T>: Future<Output = crate::Result<T>>;
  /// See [`TcpListener`].
  type TcpListener: TcpListener<TcpStream = Self::TcpStream>;
  /// See [`TcpStream`].
  type TcpStream: TcpStream<Executor = Self>;

  /// Performs a DNS resolution.
  fn lookup_host(
    host: (&str, u16),
  ) -> impl Future<Output = crate::Result<impl Iterator<Item = SocketAddr>>>;

  /// Spawns a future to run concurrently on the executor.
  fn spawn<F>(&self, future: F) -> Self::SpawnFuture<F::Output>
  where
    F: Future + Send + 'static,
    F::Output: Send + 'static;
}

/// Runs asynchronous tasks.
pub trait Runtime: Sized {
  /// Initializes a new runtime instance with optional parameters.
  fn optioned() -> crate::Result<Self>;

  /// Blocks the current thread until the provided future completes.
  fn block_on<F>(&self, future: F) -> F::Output
  where
    F: Future;
}

/// Reliable, ordered, and error-checked listening of a stream of bytes.
pub trait TcpListener: Sized {
  /// The TCP stream type produced by this listener.
  type TcpStream: TcpStream;

  /// Binds a new TCP listener to the specified address and port.
  fn bind(addr: (&str, u16), tcp_params: TcpParams) -> impl Future<Output = crate::Result<Self>>;

  /// Accepts a new incoming TCP connection.
  fn accept(
    &self,
    tcp_params: TcpParams,
  ) -> impl Future<Output = crate::Result<(Self::TcpStream, SocketAddr)>>;
}

impl TcpListener for () {
  type TcpStream = ();

  #[inline]
  async fn bind(_: (&str, u16), _: TcpParams) -> crate::Result<Self> {
    Ok(())
  }

  #[inline]
  async fn accept(&self, _: TcpParams) -> crate::Result<(Self::TcpStream, SocketAddr)> {
    Ok(((), SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from_bits(0), 0))))
  }
}

/// Reliable, ordered, and error-checked delivery of a stream of bytes.
pub trait TcpStream: Sized + Stream {
  /// The executor associated with this stream.
  type Executor;

  /// Establishes a new TCP connection to the specified address.
  fn connect(addr: (&str, u16), tcp_params: TcpParams)
  -> impl Future<Output = crate::Result<Self>>;

  /// Returns the socket address of the remote peer.
  fn peer_addr(&self) -> crate::Result<SocketAddr>;
}

impl TcpStream for () {
  type Executor = ();

  #[inline]
  async fn connect(_: (&str, u16), _: TcpParams) -> crate::Result<Self> {
    Ok(())
  }

  #[inline]
  fn peer_addr(&self) -> crate::Result<SocketAddr> {
    Ok(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from_bits(0), 0)))
  }
}

#[cfg(feature = "std")]
async fn tcp_listener_std<EX>(
  addr: (&str, u16),
  _tcp_params: TcpParams,
) -> crate::Result<std::net::TcpListener>
where
  EX: Executor,
{
  let socket_addr = crate::misc::into_rslt(EX::lookup_host(addr).await?.next())?;
  cfg_select! {
    feature = "socket2" => {
      let domain = if socket_addr.is_ipv4() {
        socket2::Domain::IPV4
      } else {
        socket2::Domain::IPV6
      };
      let socket = socket2::Socket::new(domain, socket2::Type::STREAM, None)?;
      if let Some(elem) = _tcp_params.reuse_address {
        socket.set_reuse_address(elem)?;
      }
      #[cfg(not(any(
        target_os = "cygwin",
        target_os = "illumos",
        target_os = "solaris",
        target_os = "wasi"
      )))]
      if let Some(elem) = _tcp_params.reuse_port {
        socket.set_reuse_port(elem)?;
      }
      socket.set_tcp_nodelay(_tcp_params.tcp_nodelay)?;

      // ***** THE ORDER IS IMPORTANT *****
      socket.bind(&socket_addr.into())?;
      socket.listen(_tcp_params.listen)?;
      // ***** THE ORDER IS IMPORTANT *****

      Ok(std::net::TcpListener::from(socket))
    },
    _ => Ok(std::net::TcpListener::bind(socket_addr)?)
  }
}
