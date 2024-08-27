/// URI path converted into an owned type.
#[derive(Debug)]
pub struct PathOwned<T>(
  /// Arbitrary type
  pub T,
);

/// String reference extracted from a URI path.
#[derive(Debug)]
pub struct PathStr<'uri>(
  /// Arbitrary type
  pub &'uri str,
);

/// Serializes and deserializes using `serde_json`
#[derive(Debug)]
pub struct SerdeJson<T>(
  /// Arbitrary type
  pub T,
);

#[cfg(feature = "serde_json")]
mod serde_json {
  use crate::{
    http::{
      server_framework::{Endpoint, ResponseFinalizer, SerdeJson},
      Header, KnownHeaderName, Mime, ReqResDataMut, Request, StatusCode,
    },
    misc::{serde_collect_seq_rslt, FnFut1, IterWrapper, Vector},
  };
  use serde::{de::DeserializeOwned, Serialize};

  impl<E, RRD, T> ResponseFinalizer<E, RRD> for SerdeJson<T>
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

  impl<E, RRD, I, T> ResponseFinalizer<E, RRD> for SerdeJson<IterWrapper<I>>
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

  impl<E, F, RES, RRD, T> Endpoint<SerdeJson<T>, E, RRD> for F
  where
    E: From<crate::Error>,
    F: FnFut1<SerdeJson<T>, Result = RES>,
    RES: ResponseFinalizer<E, RRD>,
    RRD: Default + ReqResDataMut<Body = Vector<u8>>,
    T: DeserializeOwned,
  {
    #[inline]
    async fn call(
      &self,
      _: &'static str,
      req: &mut Request<RRD>,
      _: [usize; 2],
    ) -> Result<StatusCode, E> {
      let elem = serde_json::from_slice(req.rrd.body()).map_err(crate::Error::from)?;
      req.rrd.body_mut().clear();
      (self)(SerdeJson(elem)).await.finalize_response(req)
    }
  }

  #[inline]
  fn push_content_type<RRD>(req: &mut Request<RRD>) -> crate::Result<()>
  where
    RRD: ReqResDataMut,
  {
    req.rrd.headers_mut().push_front(
      Header {
        is_sensitive: false,
        is_trailer: false,
        name: KnownHeaderName::ContentType.into(),
        value: Mime::Json.as_str().as_bytes(),
      },
      &[],
    )?;
    Ok(())
  }
}
