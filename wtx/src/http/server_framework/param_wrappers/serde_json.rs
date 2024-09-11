use crate::{
  http::{
    server_framework::{Endpoint, ResFinalizer},
    Header, KnownHeaderName, Mime, ReqResDataMut, Request, StatusCode,
  },
  misc::{serde_collect_seq_rslt, FnFut, FnFutWrapper, IterWrapper, Vector},
};
use serde::{de::DeserializeOwned, Serialize};

/// Serializes and deserializes using `serde_json`
#[derive(Debug)]
pub struct SerdeJson<T>(
  /// Arbitrary type
  pub T,
);

impl<CA, E, F, RA, RES, RRD, T> Endpoint<CA, E, RA, RRD> for FnFutWrapper<(SerdeJson<T>,), F>
where
  E: From<crate::Error>,
  F: FnFut<(SerdeJson<T>,), Result = RES>,
  RES: ResFinalizer<E, RRD>,
  RRD: ReqResDataMut<Body = Vector<u8>>,
  T: DeserializeOwned,
{
  #[inline]
  async fn call(
    &self,
    _: &mut CA,
    _: (u8, &[(&'static str, u8)]),
    _: &mut RA,
    req: &mut Request<RRD>,
  ) -> Result<StatusCode, E> {
    let elem = serde_json::from_slice(req.rrd.body()).map_err(crate::Error::from)?;
    req.rrd.clear();
    self.0.call((SerdeJson(elem),)).await.finalize_response(req)
  }
}

impl<E, RRD, T> ResFinalizer<E, RRD> for SerdeJson<T>
where
  E: From<crate::Error>,
  RRD: ReqResDataMut<Body = Vector<u8>>,
  T: Serialize,
{
  #[inline]
  fn finalize_response(self, req: &mut Request<RRD>) -> Result<StatusCode, E> {
    push_content_type(req).map_err(crate::Error::from)?;
    serde_json::to_writer(req.rrd.body_mut(), &self.0).map_err(crate::Error::from)?;
    Ok(StatusCode::Ok)
  }
}

impl<E, RRD, I, T> ResFinalizer<E, RRD> for SerdeJson<IterWrapper<I>>
where
  E: From<crate::Error> + From<serde_json::Error>,
  RRD: ReqResDataMut<Body = Vector<u8>>,
  I: Iterator<Item = Result<T, E>>,
  T: Serialize,
{
  #[inline]
  fn finalize_response(self, req: &mut Request<RRD>) -> Result<StatusCode, E> {
    push_content_type(req).map_err(crate::Error::from)?;
    serde_collect_seq_rslt(&mut serde_json::Serializer::new(req.rrd.body_mut()), self.0 .0)?;
    Ok(StatusCode::Ok)
  }
}

#[inline]
fn push_content_type<RRD>(req: &mut Request<RRD>) -> crate::Result<()>
where
  RRD: ReqResDataMut,
{
  req.rrd.headers_mut().push_from_iter(Header::from_name_and_value(
    KnownHeaderName::ContentType.into(),
    [Mime::Json.as_str().as_bytes()],
  ))?;
  Ok(())
}
