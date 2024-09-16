use crate::{
  http::{
    server_framework::{Endpoint, ResFinalizer},
    ReqResDataMut, Request, StatusCode,
  },
  misc::{FnFut, FnFutWrapper},
};

/// State of a connection
#[derive(Debug)]
pub struct State<'any, CA, RA, RRD> {
  /// Connection auxiliary
  pub ca: &'any mut CA,
  /// Request auxiliary
  pub ra: &'any mut RA,
  /// Request/Response Data
  pub req: &'any mut Request<RRD>,
}

impl<'any, CA, RA, RRD> State<'any, CA, RA, RRD> {
  #[inline]
  pub(crate) fn new(ca: &'any mut CA, ra: &'any mut RA, req: &'any mut Request<RRD>) -> Self {
    Self { ca, ra, req }
  }
}

impl<CA, E, F, RA, RES, RRD> Endpoint<CA, E, RA, RRD> for FnFutWrapper<(State<'_, CA, RA, RRD>,), F>
where
  F: for<'any> FnFut<(State<'any, CA, RA, RRD>,), Result = RES>,
  RES: ResFinalizer<E, RRD>,
  RRD: ReqResDataMut,
{
  #[inline]
  async fn call(
    &self,
    ca: &mut CA,
    _: (u8, &[(&'static str, u8)]),
    ra: &mut RA,
    req: &mut Request<RRD>,
  ) -> Result<StatusCode, E> {
    self.0.call((State::new(ca, ra, req),)).await.finalize_response(req)
  }
}

/// State of a connection
///
/// If used in an endpoint's argument, request data is cleared. If used as the return type of an
/// endpoint, response data is cleared.
#[derive(Debug)]
pub struct StateClean<'any, CA, RA, RRD> {
  /// Connection auxiliary
  pub ca: &'any mut CA,
  /// Request auxiliary
  pub ra: &'any mut RA,
  /// Request/Response Data
  pub req: &'any mut Request<RRD>,
}

impl<'any, CA, RA, RRD> StateClean<'any, CA, RA, RRD>
where
  RRD: ReqResDataMut,
{
  #[inline]
  pub(crate) fn new(ca: &'any mut CA, ra: &'any mut RA, req: &'any mut Request<RRD>) -> Self {
    req.rrd.clear();
    Self { ca, ra, req }
  }
}

impl<CA, E, F, RA, RES, RRD> Endpoint<CA, E, RA, RRD>
  for FnFutWrapper<(StateClean<'_, CA, RA, RRD>,), F>
where
  F: for<'any> FnFut<(StateClean<'any, CA, RA, RRD>,), Result = RES>,
  RES: ResFinalizer<E, RRD>,
  RRD: ReqResDataMut,
{
  #[inline]
  async fn call(
    &self,
    ca: &mut CA,
    _: (u8, &[(&'static str, u8)]),
    ra: &mut RA,
    req: &mut Request<RRD>,
  ) -> Result<StatusCode, E> {
    self.0.call((StateClean::new(ca, ra, req),)).await.finalize_response(req)
  }
}

impl<'any, CA, RA, RRD> From<State<'any, CA, RA, RRD>> for StateClean<'any, CA, RA, RRD> {
  #[inline]
  fn from(from: State<'any, CA, RA, RRD>) -> Self {
    Self { ca: from.ca, ra: from.ra, req: from.req }
  }
}
