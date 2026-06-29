use crate::http::{MsgBufferString, Protocol, Request};
use core::net::IpAddr;

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
pub struct AutoStream<D> {
  /// Auxiliary data
  pub data: D,
  /// data peer address
  pub peer: IpAddr,
  /// See [Protocol].
  pub protocol: Option<Protocol>,
  /// Request
  pub req: Request<MsgBufferString>,
}

impl<D> AutoStream<D> {
  /// Shortcut
  #[inline]
  pub const fn new(
    data: D,
    peer: IpAddr,
    protocol: Option<Protocol>,
    req: Request<MsgBufferString>,
  ) -> Self {
    Self { data, peer, protocol, req }
  }
}

/// HTTP stream that is manually managed by the user. For example, WebSockets over streams.
#[derive(Debug)]
pub struct ManualStream<D, S> {
  /// Auxiliary data
  pub data: D,
  /// Remote peer address
  pub peer: IpAddr,
  /// See [Protocol].
  pub protocol: Option<Protocol>,
  /// Request
  pub req: Request<MsgBufferString>,
  /// Stream
  pub stream: S,
}

impl<D, S> ManualStream<D, S> {
  /// Shortcut
  #[inline]
  pub const fn new(
    data: D,
    peer: IpAddr,
    protocol: Option<Protocol>,
    req: Request<MsgBufferString>,
    stream: S,
  ) -> Self {
    Self { data, peer, protocol, req, stream }
  }
}

/// Operation Mode Stream
pub trait OperationModeStream {
  /// Operation mode
  const OM: OperationMode;
  /// Connection Auxiliary
  type Data;

  /// Auxiliary data
  fn data(&mut self) -> &mut Self::Data;

  /// Remote peer address
  fn parts(&mut self) -> (&mut Self::Data, &mut Request<MsgBufferString>);

  /// Remote peer address
  fn peer(&self) -> &IpAddr;

  /// Request
  fn req(&mut self) -> &mut Request<MsgBufferString>;
}

impl<D> OperationModeStream for AutoStream<D> {
  const OM: OperationMode = OperationMode::Auto;
  type Data = D;

  #[inline]
  fn data(&mut self) -> &mut Self::Data {
    &mut self.data
  }

  #[inline]
  fn parts(&mut self) -> (&mut Self::Data, &mut Request<MsgBufferString>) {
    (&mut self.data, &mut self.req)
  }

  #[inline]
  fn peer(&self) -> &IpAddr {
    &self.peer
  }

  #[inline]
  fn req(&mut self) -> &mut Request<MsgBufferString> {
    &mut self.req
  }
}

impl<D, S> OperationModeStream for ManualStream<D, S> {
  const OM: OperationMode = OperationMode::Manual;
  type Data = D;

  #[inline]
  fn data(&mut self) -> &mut D {
    &mut self.data
  }

  #[inline]
  fn parts(&mut self) -> (&mut Self::Data, &mut Request<MsgBufferString>) {
    (&mut self.data, &mut self.req)
  }

  #[inline]
  fn peer(&self) -> &IpAddr {
    &self.peer
  }

  #[inline]
  fn req(&mut self) -> &mut Request<MsgBufferString> {
    &mut self.req
  }
}
