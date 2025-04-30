use crate::{
  collection::Vector,
  http::{
    AutoStream, Headers, Request, StatusCode,
    server_framework::{
      Endpoint, ResFinalizer, StateGeneric,
      arguments::{RouteMatch, manage_path},
    },
  },
  misc::{FnFut, FnFutWrapper},
};

/// String reference extracted from a URI path.
#[derive(Debug)]
pub struct PathStr<'uri>(
  /// Arbitrary type
  pub &'uri str,
);

impl<CA, E, F, RES, S, SA> Endpoint<CA, E, S, SA> for FnFutWrapper<(PathStr<'_>,), F>
where
  E: From<crate::Error>,
  F: for<'any> FnFut<(PathStr<'any>,), Result = RES>,
  RES: ResFinalizer<E>,
{
  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<CA, SA>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    auto_stream.req.rrd.clear();
    let path = manage_path(path_defs, &auto_stream.req.rrd.uri)?;
    self.0.call((PathStr(path),)).await.finalize_response(&mut auto_stream.req)
  }
}

impl<CA, E, F, RES, S, SA, const CLEAN: bool> Endpoint<CA, E, S, SA>
  for FnFutWrapper<
    (StateGeneric<'_, CA, SA, (&mut Vector<u8>, &mut Headers), CLEAN>, PathStr<'_>),
    F,
  >
where
  E: From<crate::Error>,
  F: for<'any> FnFut<
      (StateGeneric<'any, CA, SA, (&'any mut Vector<u8>, &'any mut Headers), CLEAN>, PathStr<'any>),
      Result = RES,
    >,
  RES: ResFinalizer<E>,
{
  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<CA, SA>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    let (body, headers, uri) = auto_stream.req.rrd.parts_mut();
    let mut new_req =
      Request::new(auto_stream.req.method, (body, headers), auto_stream.req.version);
    let path = manage_path(path_defs, uri)?;
    self
      .0
      .call((
        StateGeneric::new(&mut auto_stream.conn_aux, &mut auto_stream.stream_aux, &mut new_req),
        PathStr(path),
      ))
      .await
      .finalize_response(&mut auto_stream.req)
  }
}
