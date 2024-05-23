//! Optioned high-level abstraction for servers. You can use one of listed suggestions or
//! create your own.

#[cfg(all(feature = "async-send", feature = "http2", feature = "pool", feature = "tokio"))]
mod tokio_http2;
#[cfg(all(
  feature = "async-send",
  feature = "parking_lot",
  feature = "pool",
  feature = "tokio",
  feature = "web-socket-handshake"
))]
mod tokio_web_socket;

use core::marker::PhantomData;

/// See [TokioHttp2::tokio_http2].
#[cfg(all(
  feature = "async-send",
  feature = "http2",
  feature = "parking_lot",
  feature = "pool",
  feature = "tokio"
))]
pub type TokioHttp2<SB> = Server<crate::pool::Http2ServerBufferRM<crate::rng::StdRng, SB>>;
/// See [TokioWebSocket::tokio_web_socket].
#[cfg(all(
  feature = "async-send",
  feature = "pool",
  feature = "tokio",
  feature = "web-socket-handshake"
))]
pub type TokioWebSocket = Server<crate::pool::WebSocketRM>;

/// Optioned high-level abstraction for servers. You can use one of listed suggestions or
/// create your own.
///
/// Suggestions depend on the activation of different features.
#[derive(Debug)]
pub struct Server<RM> {
  rm: PhantomData<RM>,
}

#[cfg(feature = "std")]
fn _buffers_len(buffers_len_opt: Option<usize>) -> crate::Result<usize> {
  Ok(if let Some(elem) = buffers_len_opt {
    elem
  } else {
    usize::from(std::thread::available_parallelism()?).wrapping_mul(2)
  })
}
