use crate::http::{Headers, ReqResBuffer, Request};
use core::net::IpAddr;

#[cfg(all(feature = "http2", feature = "tokio"))]
/// Manual server stream backed by tokio structures.
pub type ManualServerStreamTokio<CA, SA, HB, SW> =
  ManualStream<CA, SA, crate::http2::ServerStream<crate::http2::Http2DataTokio<HB, SW, false>>>;

/// Tells how an HTTP stream should be handled.
#[derive(Debug)]
pub enum StreamMode {
  /// Automatic
  Auto,
  /// Manual
  Manual,
}

/// HTTP stream that is automatically managed by the system. In other words, all frames
/// are gathered until an end-of-stream flag is received and only then a response is sent.
#[derive(Debug)]
pub struct AutoStream<CA, SA> {
  /// Connection auxiliary
  pub ca: CA,
  /// Remote peer address
  pub peer: IpAddr,
  /// Request
  pub req: Request<ReqResBuffer>,
  /// Stream auxiliary
  pub sa: SA,
}

/// HTTP stream that is manually managed by the user. For example, WebSockets over streams.
#[derive(Debug)]
pub struct ManualStream<CA, SA, S> {
  /// Connection auxiliary
  pub ca: CA,
  /// Headers
  pub headers: Headers,
  /// Remote peer address
  pub peer: IpAddr,
  /// Stream auxiliary
  pub sa: SA,
  /// Stream
  pub stream: S,
}
