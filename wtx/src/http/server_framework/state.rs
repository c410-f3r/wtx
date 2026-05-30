use crate::{
  collection::Vector,
  http::{
    AutoStream, Headers, MsgBufferString, MsgDataMut, Request, StatusCode,
    server_framework::{Endpoint, ResFinalizer, RouteMatch},
  },
  misc::{FnFut, FnFutWrapper},
};

/// [`StateGeneric`] with original content
pub type State<'any, CA, SA, MD> = StateGeneric<'any, CA, SA, MD, false>;
/// [`StateGeneric`] with cleaned content
pub type StateClean<'any, CA, SA, MD> = StateGeneric<'any, CA, SA, MD, true>;

/// State of a connection
///
/// # If `CLEAN` is true
///
/// When used in an endpoint's argument, request data is automatically cleaned. When used as the return type of an
/// endpoint, response data is automatically cleaned.
//
// The use of generics without lifetimes make `Endpoint::auto` `!Send` when associated with an
// hypothetical `State<&mut CA, &mut SA, &mut MD>`, for whatever the reason.
#[derive(Debug)]
pub struct StateGeneric<'any, CA, SA, MD, const CLEAN: bool> {
  /// Connection auxiliary
  pub conn_aux: &'any mut CA,
  /// Request/Response Data
  pub req: &'any mut Request<MD>,
  /// Stream auxiliary
  pub stream_aux: &'any mut SA,
}

impl<'any, CA, SA, MD, const CLEAN: bool> StateGeneric<'any, CA, SA, MD, CLEAN>
where
  MD: MsgDataMut,
{
  /// Creates an instance with erased `MD` data if `CLEAN` is true.
  #[inline]
  pub fn new(conn_aux: &'any mut CA, stream_aux: &'any mut SA, req: &'any mut Request<MD>) -> Self {
    if CLEAN {
      req.clear();
    }
    Self { conn_aux, stream_aux, req }
  }
}

impl<CA, E, F, RES, S, SA, const CLEAN: bool> Endpoint<CA, E, S, SA>
  for FnFutWrapper<(StateGeneric<'_, CA, SA, MsgBufferString, CLEAN>,), F>
where
  E: From<crate::Error>,
  F: for<'any> FnFut<(StateGeneric<'any, CA, SA, MsgBufferString, CLEAN>,), Result = RES>,
  RES: ResFinalizer<E>,
{
  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<CA, SA>,
    _: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    self
      .0
      .call((StateGeneric::new(
        &mut auto_stream.conn_aux,
        &mut auto_stream.stream_aux,
        &mut auto_stream.req,
      ),))
      .await
      .finalize_response(&mut auto_stream.req)
  }
}

impl<'any, CA, SA, MD> From<State<'any, CA, SA, MD>> for StateClean<'any, CA, SA, MD>
where
  MD: MsgDataMut,
{
  #[inline]
  fn from(state: State<'any, CA, SA, MD>) -> Self {
    Self::new(state.conn_aux, state.stream_aux, state.req)
  }
}

impl<'any, CA, SA, MD, const CLEAN: bool> From<&'any mut (CA, SA, Request<MD>)>
  for StateGeneric<'any, CA, SA, MD, CLEAN>
where
  MD: MsgDataMut,
{
  #[inline]
  fn from((conn_aux, stream_aux, req): &'any mut (CA, SA, Request<MD>)) -> Self {
    Self::new(conn_aux, stream_aux, req)
  }
}

impl<'any, CA, SA, MD, const CLEAN: bool> From<&'any mut (&'any mut CA, &'any mut SA, Request<MD>)>
  for StateGeneric<'any, CA, SA, MD, CLEAN>
where
  MD: MsgDataMut,
{
  #[inline]
  fn from(
    (conn_aux, stream_aux, req): &'any mut (&'any mut CA, &'any mut SA, Request<MD>),
  ) -> Self {
    Self::new(conn_aux, stream_aux, req)
  }
}

/// Owned state used in testing environments.
#[derive(Debug)]
pub struct StateTest<CA, SA, MD> {
  /// Connection auxiliary
  pub conn_aux: CA,
  /// Request/Response Data
  pub req: Request<MD>,
  /// Stream auxiliary
  pub stream_aux: SA,
}

impl<CA, SA, MD> StateTest<CA, SA, MD> {
  /// Mutable parts
  #[inline]
  pub const fn parts_mut(&mut self) -> (&mut CA, &mut SA, &mut Request<MD>) {
    (&mut self.conn_aux, &mut self.stream_aux, &mut self.req)
  }

  /// Returns a new [`StateGeneric`].
  #[inline]
  pub const fn state<const CLEAR: bool>(&mut self) -> StateGeneric<'_, CA, SA, MD, CLEAR> {
    StateGeneric {
      conn_aux: &mut self.conn_aux,
      req: &mut self.req,
      stream_aux: &mut self.stream_aux,
    }
  }
}

impl<CA, SA> StateTest<CA, SA, MsgBufferString> {
  /// Mutable parts with a modified request that only contains data and headers.
  #[inline]
  pub const fn parts_mut_with_body_and_headers(
    &mut self,
  ) -> (&mut CA, &mut SA, Request<(&mut Vector<u8>, &mut Headers)>) {
    (
      &mut self.conn_aux,
      &mut self.stream_aux,
      Request {
        method: self.req.method,
        msg_data: (&mut self.req.msg_data.body, &mut self.req.msg_data.headers),
      },
    )
  }
}
