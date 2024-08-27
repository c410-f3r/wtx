mod with_path;

use crate::{
  http::{server_framework::ResponseFinalizer, Request, StatusCode},
  misc::{FnFut0, FnFut1},
};
use core::future::Future;

/// Path function
pub trait Endpoint<A, E, RRD> {
  /// Generates a response.
  fn call(
    &self,
    matching_path: &'static str,
    req: &mut Request<RRD>,
    req_path_indcs: [usize; 2],
  ) -> impl Future<Output = Result<StatusCode, E>>;
}

impl<E, F, RES, RRD> Endpoint<(), E, RRD> for F
where
  F: FnFut0<Result = RES>,
  RES: ResponseFinalizer<E, RRD>,
{
  #[inline]
  async fn call(
    &self,
    _: &'static str,
    req: &mut Request<RRD>,
    _: [usize; 2],
  ) -> Result<StatusCode, E> {
    (self)().await.finalize_response(req)
  }
}

impl<E, F, RES, RRD> Endpoint<&mut RRD, E, RRD> for F
where
  F: for<'any> FnFut1<&'any mut RRD, Result = RES>,
  RES: ResponseFinalizer<E, RRD>,
{
  #[inline]
  async fn call(
    &self,
    _: &'static str,
    req: &mut Request<RRD>,
    _: [usize; 2],
  ) -> Result<StatusCode, E> {
    (self)(&mut req.rrd).await.finalize_response(req)
  }
}

impl<E, F, RES, RRD> Endpoint<&mut Request<RRD>, E, RRD> for F
where
  F: for<'any> FnFut1<&'any mut Request<RRD>, Result = RES>,
  RES: ResponseFinalizer<E, RRD>,
{
  #[inline]
  async fn call(
    &self,
    _: &'static str,
    req: &mut Request<RRD>,
    _: [usize; 2],
  ) -> Result<StatusCode, E> {
    (self)(req).await.finalize_response(req)
  }
}

#[cfg(feature = "grpc")]
mod grpc {
  use crate::{
    grpc::{GrpcStatusCode, ServerData},
    http::{
      server_framework::Endpoint, Header, KnownHeaderName, Mime, ReqResDataMut, Request, StatusCode,
    },
    misc::FnFut2,
  };

  impl<DRSR, E, F, RRD> Endpoint<(&mut Request<RRD>, ServerData<DRSR>), E, RRD> for F
  where
    DRSR: Default,
    E: From<crate::Error>,
    F: for<'any> FnFut2<
      &'any mut Request<RRD>,
      ServerData<DRSR>,
      Result = Result<(StatusCode, GrpcStatusCode), E>,
    >,
    RRD: ReqResDataMut,
  {
    #[inline]
    async fn call(
      &self,
      _: &'static str,
      req: &mut Request<RRD>,
      _: [usize; 2],
    ) -> Result<StatusCode, E> {
      let (status_code, gsc) = (self)(req, ServerData::new(DRSR::default())).await?;
      req.rrd.headers_mut().clear();
      req.rrd.headers_mut().push_front(
        Header {
          is_sensitive: false,
          is_trailer: false,
          name: KnownHeaderName::ContentType.into(),
          value: Mime::Grpc.as_str().as_bytes(),
        },
        &[],
      )?;
      req.rrd.headers_mut().push_front(
        Header {
          is_sensitive: false,
          is_trailer: true,
          name: b"grpc-status",
          value: gsc.number_as_str().as_bytes(),
        },
        &[],
      )?;
      Ok(status_code)
    }
  }
}
