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

#[cfg(feature = "std")]
use crate::net::ToSocketAddrs;
use crate::net::{TcpListener, TcpStream};
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

impl ExecutorTy {
  /// Returns `true` if the instance is [`ExecutorTy::Std`].
  #[inline]
  #[must_use]
  pub const fn is_std(&self) -> bool {
    matches!(self, Self::Std)
  }
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
async fn tcp_listener_std<A, EX>(
  addr: A,
  executor: &EX,
  _tcp_params: crate::net::TcpParams,
) -> crate::Result<std::net::TcpListener>
where
  A: ToSocketAddrs,
  EX: Executor,
{
  let fun = async |socket_addr: SocketAddr| {
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
  };
  resolve_addrs(addr, executor, fun).await
}

#[cfg(feature = "std")]
async fn resolve_addrs<A, EX, T>(
  addr: A,
  executor: &EX,
  mut cb: impl AsyncFnMut(SocketAddr) -> crate::Result<T>,
) -> crate::Result<T>
where
  A: ToSocketAddrs,
  EX: Executor,
{
  let mut last_err = None;
  for socket_addr in addr.to_socket_addrs(executor).await? {
    match cb(socket_addr).await {
      Ok(el) => return Ok(el),
      Err(err) => last_err = Some(err),
    }
  }
  Err(last_err.unwrap_or_else(|| ExecutorError::InvalidResolvedAddress.into()))
}
