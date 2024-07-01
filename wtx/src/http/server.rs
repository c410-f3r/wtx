//! Optioned high-level abstraction for servers. You can use one of listed suggestions or
//! create your own.

#[cfg(all(feature = "http2", feature = "tokio"))]
mod tokio_http2;
#[cfg(all(feature = "pool", feature = "tokio", feature = "web-socket-handshake"))]
mod tokio_web_socket;

/// Optioned high-level abstraction for servers.
#[derive(Debug)]
pub struct OptionedServer;

#[cfg(feature = "std")]
fn _buffers_len(buffers_len_opt: Option<usize>) -> crate::Result<usize> {
  Ok(if let Some(elem) = buffers_len_opt {
    elem
  } else {
    usize::from(std::thread::available_parallelism()?).wrapping_mul(2)
  })
}
