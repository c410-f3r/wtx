use crate::{
  http::{
    server_framework::{Endpoint, ResFinalizer},
    ReqResDataMut, Request, StatusCode,
  },
  misc::{FnFut, FnFutWrapper},
};

/// [`StateGeneric`] with original content
pub type State<'any, CA, RA, RRD> = StateGeneric<'any, CA, RA, RRD, false>;
/// [`StateGeneric`] with cleaned content
pub type StateClean<'any, CA, RA, RRD> = StateGeneric<'any, CA, RA, RRD, true>;

/// State of a connection
///
/// # If `CLEAN` is true
///
/// When used in an endpoint's argument, request data is automatically cleaned. When used as the return type of an
/// endpoint, response data is automatically cleaned.
#[derive(Debug)]
pub struct StateGeneric<'any, CA, RA, RRD, const CLEAN: bool> {
  /// Connection auxiliary
  pub ca: &'any mut CA,
  /// Request auxiliary
  pub ra: &'any mut RA,
  /// Request/Response Data
  pub req: &'any mut Request<RRD>,
}

impl<'any, CA, RA, RRD, const CLEAN: bool> StateGeneric<'any, CA, RA, RRD, CLEAN>
where
  RRD: ReqResDataMut,
{
  /// Creates an instance with erased `RRD` data if `CLEAN` is true.
  #[inline]
  pub fn new(ca: &'any mut CA, ra: &'any mut RA, req: &'any mut Request<RRD>) -> Self {
    if CLEAN {
      req.rrd.clear();
    }
    Self { ca, ra, req }
  }
}

impl<CA, E, F, RA, RES, RRD, const CLEAN: bool> Endpoint<CA, E, RA, RRD>
  for FnFutWrapper<(StateGeneric<'_, CA, RA, RRD, CLEAN>,), F>
where
  F: for<'any> FnFut<(StateGeneric<'any, CA, RA, RRD, CLEAN>,), Result = RES>,
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
    self.0.call((StateGeneric::new(ca, ra, req),)).await.finalize_response(req)
  }
}
