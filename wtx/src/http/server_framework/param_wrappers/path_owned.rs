use crate::{
  http::{
    server_framework::{param_wrappers::manage_path, Endpoint, ResFinalizer, State, StateClean},
    ReqResDataMut, Request, StatusCode,
  },
  misc::{FnFut, FnFutWrapper},
};
use core::str::FromStr;

/// URI path converted into an owned type.
#[derive(Debug)]
pub struct PathOwned<T>(
  /// Arbitrary type
  pub T,
);

impl<CA, E, F, P, RA, RES, RRD> Endpoint<CA, E, RA, RRD> for FnFutWrapper<(PathOwned<P>,), F>
where
  E: From<crate::Error>,
  P: FromStr,
  P::Err: Into<crate::Error>,
  F: for<'any> FnFut<(PathOwned<P>,), Result = RES>,
  RES: ResFinalizer<E, RRD>,
  RRD: ReqResDataMut,
{
  #[inline]
  async fn call(
    &self,
    _: &mut CA,
    path_defs: (u8, &[(&'static str, u8)]),
    _: &mut RA,
    req: &mut Request<RRD>,
  ) -> Result<StatusCode, E> {
    req.rrd.clear();
    let uri = req.rrd.uri();
    let path = manage_path(path_defs, &uri).map_err(From::from)?;
    let path_owned = PathOwned(P::from_str(path).map_err(Into::into)?);
    self.0.call((path_owned,)).await.finalize_response(req)
  }
}

impl<CA, E, F, P, RA, RES, RRD> Endpoint<CA, E, RA, RRD>
  for FnFutWrapper<(State<'_, CA, RA, RRD>, PathOwned<P>), F>
where
  E: From<crate::Error>,
  P: FromStr,
  P::Err: Into<crate::Error>,
  F: for<'any> FnFut<(State<'any, CA, RA, RRD>, PathOwned<P>), Result = RES>,
  RES: ResFinalizer<E, RRD>,
  RRD: ReqResDataMut,
{
  #[inline]
  async fn call(
    &self,
    ca: &mut CA,
    path_defs: (u8, &[(&'static str, u8)]),
    ra: &mut RA,
    req: &mut Request<RRD>,
  ) -> Result<StatusCode, E> {
    let uri = req.rrd.uri();
    let path = manage_path(path_defs, &uri).map_err(From::from)?;
    let path_owned = PathOwned(P::from_str(path).map_err(Into::into)?);
    self.0.call((State::new(ca, ra, req), path_owned)).await.finalize_response(req)
  }
}

impl<CA, E, F, P, RA, RES, RRD> Endpoint<CA, E, RA, RRD>
  for FnFutWrapper<(StateClean<'_, CA, RA, RRD>, PathOwned<P>), F>
where
  E: From<crate::Error>,
  P: FromStr,
  P::Err: Into<crate::Error>,
  F: for<'any> FnFut<(StateClean<'any, CA, RA, RRD>, PathOwned<P>), Result = RES>,
  RES: ResFinalizer<E, RRD>,
  RRD: ReqResDataMut,
{
  #[inline]
  async fn call(
    &self,
    ca: &mut CA,
    path_defs: (u8, &[(&'static str, u8)]),
    ra: &mut RA,
    req: &mut Request<RRD>,
  ) -> Result<StatusCode, E> {
    let uri = req.rrd.uri();
    let path = manage_path(path_defs, &uri).map_err(From::from)?;
    let path_owned = PathOwned(P::from_str(path).map_err(Into::into)?);
    self.0.call((StateClean::new(ca, ra, req), path_owned)).await.finalize_response(req)
  }
}