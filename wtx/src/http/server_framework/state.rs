use crate::{
  http::{
    AutoStream, ReqResBuffer, ReqResDataMut, Request, StatusCode,
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
#[derive(Debug)]
pub struct StateGeneric<'any, CA, SA, RRD, const CLEAN: bool> {
  /// Connection auxiliary
  pub conn_aux: &'any mut CA,
  /// Request/Response Data
  pub req: &'any mut Request<RRD>,
  /// Request auxiliary
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
      req.rrd.clear();
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
