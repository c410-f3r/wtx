use crate::{
  http::{
    AutoStream, Header, KnownHeaderName, Mime, ReqResBuffer, Request, StatusCode,
    server_framework::{Endpoint, ResFinalizer, RouteMatch, StateGeneric},
  },
  misc::{FnFut, FnFutWrapper, LeaseMut},
};
use serde::{Serialize, de::DeserializeOwned};

/// Serializes and deserializes using `serde_json`
#[derive(Debug)]
pub struct SerdeJson<T>(
  /// Arbitrary type
  pub T,
);

impl<CA, E, F, RES, S, SA, T> Endpoint<CA, E, S, SA> for FnFutWrapper<(SerdeJson<T>,), F>
where
  E: From<crate::Error>,
  F: FnFut<(SerdeJson<T>,), Result = RES>,
  RES: ResFinalizer<E>,
  T: DeserializeOwned,
{
  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<CA, SA>,
    _: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    let elem =
      serde_json::from_slice(&auto_stream.req.rrd.lease_mut().body).map_err(crate::Error::from)?;
    auto_stream.req.rrd.lease_mut().clear();
    self.0.call((SerdeJson(elem),)).await.finalize_response(&mut auto_stream.req)
  }
}

impl<CA, E, F, RES, S, SA, T, const CLEAN: bool> Endpoint<CA, E, S, SA>
  for FnFutWrapper<(StateGeneric<'_, CA, SA, ReqResBuffer, CLEAN>, SerdeJson<T>), F>
where
  E: From<crate::Error>,
  F: for<'any> FnFut<(StateGeneric<'any, CA, SA, ReqResBuffer, CLEAN>, SerdeJson<T>), Result = RES>,
  RES: ResFinalizer<E>,
  T: DeserializeOwned,
{
  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<CA, SA>,
    _: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    let elem =
      serde_json::from_slice(&auto_stream.req.rrd.lease_mut().body).map_err(crate::Error::from)?;
    auto_stream.req.rrd.lease_mut().clear();
    self
      .0
      .call((
        StateGeneric::new(
          &mut auto_stream.conn_aux,
          &mut auto_stream.stream_aux,
          &mut auto_stream.req,
        ),
        SerdeJson(elem),
      ))
      .await
      .finalize_response(&mut auto_stream.req)
  }
}

impl<E, T> ResFinalizer<E> for SerdeJson<T>
where
  E: From<crate::Error>,
  T: Serialize,
{
  #[inline]
  fn finalize_response(self, req: &mut Request<ReqResBuffer>) -> Result<StatusCode, E> {
    push_content_type(req)?;
    serde_json::to_writer(&mut req.rrd.lease_mut().body, &self.0).map_err(crate::Error::from)?;
    Ok(StatusCode::Ok)
  }
}

#[inline]
fn push_content_type(req: &mut Request<ReqResBuffer>) -> crate::Result<()> {
  req.rrd.lease_mut().headers.push_from_iter(Header::from_name_and_value(
    KnownHeaderName::ContentType.into(),
    [Mime::ApplicationJson.as_str().as_bytes()],
  ))?;
  Ok(())
}
