use crate::{
  http::{
    AutoStream, ReqBuilder, ReqResBuffer, Request, StatusCode,
    server_framework::{Endpoint, ResFinalizer, RouteMatch, StateGeneric},
  },
  misc::{FnFut, FnFutWrapper, LeaseMut, serde_json_deserialize_from_slice},
};
use serde::{Serialize, de::DeserializeOwned};

/// Serializes and deserializes using `serde_json`
#[derive(Debug)]
pub struct SerdeJsonOwned<T>(
  /// Arbitrary type
  pub T,
);

impl<CA, E, F, RES, S, SA, T> Endpoint<CA, E, S, SA> for FnFutWrapper<(SerdeJsonOwned<T>,), F>
where
  E: From<crate::Error>,
  F: FnFut<(SerdeJsonOwned<T>,), Result = RES>,
  RES: ResFinalizer<E>,
  T: DeserializeOwned,
{
  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<CA, SA>,
    _: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    let body = &auto_stream.req.rrd.lease_mut().body;
    let elem = serde_json_deserialize_from_slice(body).map_err(crate::Error::from)?;
    auto_stream.req.rrd.lease_mut().clear();
    self.0.call((SerdeJsonOwned(elem),)).await.finalize_response(&mut auto_stream.req)
  }
}

impl<CA, E, F, RES, S, SA, T, const CLEAN: bool> Endpoint<CA, E, S, SA>
  for FnFutWrapper<(StateGeneric<'_, CA, SA, ReqResBuffer, CLEAN>, SerdeJsonOwned<T>), F>
where
  E: From<crate::Error>,
  F: for<'any> FnFut<
      (StateGeneric<'any, CA, SA, ReqResBuffer, CLEAN>, SerdeJsonOwned<T>),
      Result = RES,
    >,
  RES: ResFinalizer<E>,
  T: DeserializeOwned,
{
  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<CA, SA>,
    _: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    let body = &auto_stream.req.rrd.lease_mut().body;
    let elem = serde_json_deserialize_from_slice(body).map_err(crate::Error::from)?;
    auto_stream.req.rrd.lease_mut().clear();
    self
      .0
      .call((
        StateGeneric::new(
          &mut auto_stream.conn_aux,
          &mut auto_stream.stream_aux,
          &mut auto_stream.req,
        ),
        SerdeJsonOwned(elem),
      ))
      .await
      .finalize_response(&mut auto_stream.req)
  }
}

impl<E, T> ResFinalizer<E> for SerdeJsonOwned<T>
where
  E: From<crate::Error>,
  T: Serialize,
{
  #[inline]
  fn finalize_response(self, req: &mut Request<ReqResBuffer>) -> Result<StatusCode, E> {
    req.clear();
    let _ = ReqBuilder::from_req_mut(req).serde_json(&self.0)?;
    Ok(StatusCode::Ok)
  }
}
