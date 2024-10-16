use crate::{
  http::{
    server_framework::{param_wrappers::manage_path, Endpoint, ResFinalizer, StateGeneric},
    Headers, ReqResBuffer, Request, StatusCode,
  },
  misc::{FnFut, FnFutWrapper, Vector},
};

/// String reference extracted from a URI path.
#[derive(Debug)]
pub struct PathStr<'uri>(
  /// Arbitrary type
  pub &'uri str,
);

impl<CA, E, F, RA, RES> Endpoint<CA, E, RA> for FnFutWrapper<(PathStr<'_>,), F>
where
  E: From<crate::Error>,
  F: for<'any> FnFut<(PathStr<'any>,), Result = RES>,
  RES: ResFinalizer<E>,
{
  #[inline]
  async fn call(
    &self,
    _: &mut CA,
    path_defs: (u8, &[(&'static str, u8)]),
    _: &mut RA,
    req: &mut Request<ReqResBuffer>,
  ) -> Result<StatusCode, E> {
    req.rrd.clear();
    let path = manage_path(path_defs, &req.rrd.uri).map_err(From::from)?;
    self.0.call((PathStr(path),)).await.finalize_response(req)
  }
}

impl<CA, E, F, RA, RES, const CLEAN: bool> Endpoint<CA, E, RA>
  for FnFutWrapper<
    (StateGeneric<'_, CA, RA, (&mut Vector<u8>, &mut Headers), CLEAN>, PathStr<'_>),
    F,
  >
where
  E: From<crate::Error>,
  F: for<'any> FnFut<
    (StateGeneric<'any, CA, RA, (&'any mut Vector<u8>, &'any mut Headers), CLEAN>, PathStr<'any>),
    Result = RES,
  >,
  RES: ResFinalizer<E>,
{
  #[inline]
  async fn call(
    &self,
    ca: &mut CA,
    path_defs: (u8, &[(&'static str, u8)]),
    ra: &mut RA,
    req: &mut Request<ReqResBuffer>,
  ) -> Result<StatusCode, E> {
    let (body, headers, uri) = req.rrd.parts_mut();
    let mut new_req = Request::_new(req.method, (body, headers), req.version);
    let path = manage_path(path_defs, uri).map_err(From::from)?;
    self
      .0
      .call((StateGeneric::new(ca, ra, &mut new_req), PathStr(path)))
      .await
      .finalize_response(req)
  }
}
