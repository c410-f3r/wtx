//! Groups elements related to async operations.

#![allow(clippy::disallowed_types, reason = "traits require the `Arc` from std")]

mod executor_error;
mod no_std_runtime;
#[cfg(feature = "std")]
mod std_executor;
#[cfg(feature = "std")]
mod std_runtime;
#[cfg(feature = "tokio")]
mod tokio_executor;

use crate::stream::{TcpListener, TcpStream};
use core::net::SocketAddr;
pub use executor_error::ExecutorError;
pub use no_std_runtime::NoStdRuntime;
#[cfg(feature = "tokio")]
pub use tokio_executor::{TokioExecutor, TokioSpawnFutureFuture};
#[cfg(feature = "std")]
pub use {
  std_executor::{StdExecutor, StdSpawnFuture, StdSpawnLocalFuture},
  std_runtime::{SpawnFuture, StdRuntime},
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
pub trait Executor: Default {
  /// See [`ExecutorTy`].
  const TY: ExecutorTy;

  /// `!Send` bound, less overhead and thread affinity.
  type LocalRuntime: Runtime;
  /// Future of [`Self::spawn`].
  type SpawnFuture<T>: Future<Output = crate::Result<T>>;
  /// Future of [`Self::spawn_local`].
  type SpawnLocalFuture<T>: Future<Output = crate::Result<T>>;
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

  /// `!Send` version of [`Self::spawn`]
  fn spawn_local<F>(&self, future: F) -> Self::SpawnLocalFuture<F::Output>
  where
    F: Future + 'static,
    F::Output: 'static;
}

/// Runs asynchronous tasks.
pub trait Runtime: Sized {
  /// Initializes a new runtime instance with optional parameters.
  fn new() -> crate::Result<Self>;

  /// Blocks the current thread until the provided future completes.
  fn block_on<F>(&self, future: F) -> F::Output
  where
    F: Future;
}

#[cfg(feature = "std")]
async fn tcp_listener_std<EX>(
  addr: (&str, u16),
  _tcp_params: crate::misc::TcpParams,
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
