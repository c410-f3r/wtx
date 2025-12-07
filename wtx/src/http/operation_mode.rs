use crate::http::{Protocol, ReqResBuffer, Request};
use core::net::IpAddr;

#[cfg(feature = "http2")]
/// Manual server stream backed by tokio structures.
pub type ManualServerStream<CA, HB, SA, SW> =
  ManualStream<CA, crate::http2::ServerStream<HB, SW>, SA>;

/// Tells how an HTTP stream should be handled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationMode {
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
  pub conn_aux: CA,
  /// Remote peer address
  pub peer: IpAddr,
  /// See [Protocol].
  pub protocol: Option<Protocol>,
  /// Request
  pub req: Request<ReqResBuffer>,
  /// Stream auxiliary
  pub stream_aux: SA,
}

/// HTTP stream that is manually managed by the user. For example, WebSockets over streams.
#[derive(Debug)]
pub struct ManualStream<CA, S, SA> {
  /// Connection auxiliary
  pub conn_aux: CA,
  /// Remote peer address
  pub peer: IpAddr,
  /// See [Protocol].
  pub protocol: Option<Protocol>,
  /// Request
  pub req: Request<ReqResBuffer>,
  /// Stream
  pub stream: S,
  /// Stream auxiliary
  pub stream_aux: SA,
}

/// Operation Mode Stream
pub trait OperationModeStream {
  /// Operation mode
  const OM: OperationMode;
  /// Connection Auxiliary
  type ConnAux;
  /// Stream Auxiliary
  type StreamAux;

  /// Connection auxiliary
  fn conn_aux(&mut self) -> &mut Self::ConnAux;

  /// Remote peer address
  fn parts(&mut self) -> (&mut Self::ConnAux, &mut Request<ReqResBuffer>, &mut Self::StreamAux);

  /// Remote peer address
  fn peer(&self) -> &IpAddr;

  /// Request
  fn req(&mut self) -> &mut Request<ReqResBuffer>;

  /// Stream auxiliary
  fn stream_aux(&mut self) -> &mut Self::StreamAux;
}

impl<CA, SA> OperationModeStream for AutoStream<CA, SA> {
  const OM: OperationMode = OperationMode::Auto;
  type ConnAux = CA;
  type StreamAux = SA;

  #[inline]
  fn conn_aux(&mut self) -> &mut Self::ConnAux {
    &mut self.conn_aux
  }

  #[inline]
  fn parts(&mut self) -> (&mut Self::ConnAux, &mut Request<ReqResBuffer>, &mut Self::StreamAux) {
    (&mut self.conn_aux, &mut self.req, &mut self.stream_aux)
  }

  #[inline]
  fn peer(&self) -> &IpAddr {
    &self.peer
  }

  #[inline]
  fn req(&mut self) -> &mut Request<ReqResBuffer> {
    &mut self.req
  }

  #[inline]
  fn stream_aux(&mut self) -> &mut Self::StreamAux {
    &mut self.stream_aux
  }
}

impl<CA, S, SA> OperationModeStream for ManualStream<CA, S, SA> {
  const OM: OperationMode = OperationMode::Manual;
  type ConnAux = CA;
  type StreamAux = SA;

  #[inline]
  fn conn_aux(&mut self) -> &mut CA {
    &mut self.conn_aux
  }

  #[inline]
  fn parts(&mut self) -> (&mut Self::ConnAux, &mut Request<ReqResBuffer>, &mut Self::StreamAux) {
    (&mut self.conn_aux, &mut self.req, &mut self.stream_aux)
  }

  #[inline]
  fn peer(&self) -> &IpAddr {
    &self.peer
  }

  #[inline]
  fn req(&mut self) -> &mut Request<ReqResBuffer> {
    &mut self.req
  }

  #[inline]
  fn stream_aux(&mut self) -> &mut SA {
    &mut self.stream_aux
  }
}
