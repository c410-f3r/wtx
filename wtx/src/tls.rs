#![allow(dead_code, missing_docs, reason = "development")]

mod acceptor;
mod connector;
mod handshake;
mod item;
mod state;
mod tls_error;
mod tls_stream;
mod trust_anchor;

pub use acceptor::{acceptor_backend::AcceptorBackend, Acceptor};
pub use connector::{connector_backend::ConnectorBackend, Connector};
pub use tls_error::TlsError;
pub use tls_stream::TlsStream;
pub use trust_anchor::TrustAnchor;

/// Marker used to indicate the operations with `rustls`
#[cfg(feature = "rustls")]
#[derive(Debug)]
pub struct Rustls;

#[inline]
fn _invalid_input_err<E>(err: E) -> std::io::Error
where
  E: Into<alloc::boxed::Box<dyn core::error::Error + Send + Sync>>,
{
  std::io::Error::new(std::io::ErrorKind::InvalidInput, err)
}
