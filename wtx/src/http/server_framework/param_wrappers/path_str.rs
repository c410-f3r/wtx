use crate::{
  http::{
    server_framework::{param_wrappers::manage_path, Endpoint, ResFinalizer},
    ReqResDataMut, Request, StatusCode,
  },
  misc::{FnFut, FnFutWrapper},
};

/// String reference extracted from a URI path.
#[derive(Debug)]
pub struct PathStr<'uri>(
  /// Arbitrary type
  pub &'uri str,
);

impl<CA, E, F, RA, RES, RRD> Endpoint<CA, E, RA, RRD> for FnFutWrapper<(PathStr<'_>,), F>
where
  E: From<crate::Error>,
  F: for<'any> FnFut<(PathStr<'any>,), Result = RES>,
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
    self.0.call((PathStr(path),)).await.finalize_response(req)
  }
}
