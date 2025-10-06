use crate::{
  collection::Vector,
  http::{
    AutoStream, Headers, ReqResBuffer, ReqResDataMut, Request, StatusCode,
    server_framework::{Endpoint, ResFinalizer, RouteMatch},
  },
  misc::{FnFut, FnFutWrapper},
};

/// [`StateGeneric`] with original content
pub type State<'any, CA, SA, RRD> = StateGeneric<'any, CA, SA, RRD, false>;
/// [`StateGeneric`] with cleaned content
pub type StateClean<'any, CA, SA, RRD> = StateGeneric<'any, CA, SA, RRD, true>;

/// State of a connection
///
/// # If `CLEAN` is true
///
/// When used in an endpoint's argument, request data is automatically cleaned. When used as the return type of an
/// endpoint, response data is automatically cleaned.
//
// The use of generics without lifetimes make `Endpoint::auto` `!Send` when associated with an
// hypothetical `State<&mut CA, &mut SA, &mut RRD>`, for whatever the reason.
#[derive(Debug)]
pub struct StateGeneric<'any, CA, SA, RRD, const CLEAN: bool> {
  /// Connection auxiliary
  pub conn_aux: &'any mut CA,
  /// Request/Response Data
  pub req: &'any mut Request<RRD>,
  /// Stream auxiliary
  pub stream_aux: &'any mut SA,
}

impl<'any, CA, SA, RRD, const CLEAN: bool> StateGeneric<'any, CA, SA, RRD, CLEAN>
where
  RRD: ReqResDataMut,
{
  /// Creates an instance with erased `RRD` data if `CLEAN` is true.
  #[inline]
  pub fn new(
    conn_aux: &'any mut CA,
    stream_aux: &'any mut SA,
    req: &'any mut Request<RRD>,
  ) -> Self {
    if CLEAN {
      req.clear();
    }
    Self { conn_aux, stream_aux, req }
  }
}

impl<CA, E, F, RES, S, SA, const CLEAN: bool> Endpoint<CA, E, S, SA>
  for FnFutWrapper<(StateGeneric<'_, CA, SA, ReqResBuffer, CLEAN>,), F>
where
  E: From<crate::Error>,
  F: for<'any> FnFut<(StateGeneric<'any, CA, SA, ReqResBuffer, CLEAN>,), Result = RES>,
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

impl<'any, CA, SA, RRD> From<State<'any, CA, SA, RRD>> for StateClean<'any, CA, SA, RRD>
where
  RRD: ReqResDataMut,
{
  #[inline]
  fn from(state: State<'any, CA, SA, RRD>) -> Self {
    Self::new(state.conn_aux, state.stream_aux, state.req)
  }
}

impl<'any, CA, SA, RRD, const CLEAN: bool> From<&'any mut (CA, SA, Request<RRD>)>
  for StateGeneric<'any, CA, SA, RRD, CLEAN>
where
  RRD: ReqResDataMut,
{
  #[inline]
  fn from((conn_aux, stream_aux, req): &'any mut (CA, SA, Request<RRD>)) -> Self {
    Self::new(conn_aux, stream_aux, req)
  }
}

impl<'any, CA, SA, RRD, const CLEAN: bool>
  From<&'any mut (&'any mut CA, &'any mut SA, Request<RRD>)>
  for StateGeneric<'any, CA, SA, RRD, CLEAN>
where
  RRD: ReqResDataMut,
{
  #[inline]
  fn from(
    (conn_aux, stream_aux, req): &'any mut (&'any mut CA, &'any mut SA, Request<RRD>),
  ) -> Self {
    Self::new(conn_aux, stream_aux, req)
  }
}

/// Owned state used in testing environments.
#[derive(Debug)]
pub struct StateTest<CA, SA, RRD> {
  /// Connection auxiliary
  pub conn_aux: CA,
  /// Request/Response Data
  pub req: Request<RRD>,
  /// Stream auxiliary
  pub stream_aux: SA,
}

impl<CA, SA, RRD> StateTest<CA, SA, RRD> {
  /// Mutable parts
  #[inline]
  pub const fn parts_mut(&mut self) -> (&mut CA, &mut SA, &mut Request<RRD>) {
    (&mut self.conn_aux, &mut self.stream_aux, &mut self.req)
  }

  /// Returns a new [`StateGeneric`].
  #[inline]
  pub const fn state<const CLEAR: bool>(&mut self) -> StateGeneric<'_, CA, SA, RRD, CLEAR> {
    StateGeneric {
      conn_aux: &mut self.conn_aux,
      req: &mut self.req,
      stream_aux: &mut self.stream_aux,
    }
  }
}

impl<CA, SA> StateTest<CA, SA, ReqResBuffer> {
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
        rrd: (&mut self.req.rrd.body, &mut self.req.rrd.headers),
        version: self.req.version,
      },
    )
  }
}
