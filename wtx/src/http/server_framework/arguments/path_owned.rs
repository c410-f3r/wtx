use crate::{
  http::{
    AutoStream, ReqResBuffer, StatusCode,
    server_framework::{Endpoint, ResFinalizer, RouteMatch, StateGeneric, arguments::manage_path},
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

impl<CA, E, F, P, RES, S, SA> Endpoint<CA, E, S, SA> for FnFutWrapper<(PathOwned<P>,), F>
where
  E: From<crate::Error>,
  P: FromStr,
  P::Err: Into<crate::Error>,
  F: for<'any> FnFut<(PathOwned<P>,), Result = RES>,
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
    let path_owned = PathOwned(P::from_str(path).map_err(Into::into)?);
    self.0.call((path_owned,)).await.finalize_response(&mut auto_stream.req)
  }
}

impl<CA, E, F, P, RES, S, SA, const CLEAN: bool> Endpoint<CA, E, S, SA>
  for FnFutWrapper<(StateGeneric<'_, CA, SA, ReqResBuffer, CLEAN>, PathOwned<P>), F>
where
  E: From<crate::Error>,
  P: FromStr,
  P::Err: Into<crate::Error>,
  F: for<'any> FnFut<(StateGeneric<'any, CA, SA, ReqResBuffer, CLEAN>, PathOwned<P>), Result = RES>,
  RES: ResFinalizer<E>,
{
  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<CA, SA>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    let path = manage_path(path_defs, &auto_stream.req.rrd.uri)?;
    let path_owned = PathOwned(P::from_str(path).map_err(Into::into)?);
    self
      .0
      .call((
        StateGeneric::new(
          &mut auto_stream.conn_aux,
          &mut auto_stream.stream_aux,
          &mut auto_stream.req,
        ),
        path_owned,
      ))
      .await
      .finalize_response(&mut auto_stream.req)
  }
}
