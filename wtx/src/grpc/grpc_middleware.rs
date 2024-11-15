use crate::{
  grpc::GrpcManager,
  http::{
    server_framework::Middleware, Header, KnownHeaderName, Mime, ReqResBuffer, Request, Response,
    StatusCode,
  },
};
use core::ops::ControlFlow;

/// Applies gRPC headers
#[derive(Debug)]
pub struct GrpcMiddleware;

impl<CA, DRSR, E> Middleware<CA, E, GrpcManager<DRSR>> for GrpcMiddleware
where
  E: From<crate::Error>,
{
  type Aux = ();

  #[inline]
  fn aux(&self) -> Self::Aux {
    ()
  }

  #[inline]
  async fn req(
    &self,
    _: &mut CA,
    _: &mut Self::Aux,
    req: &mut Request<ReqResBuffer>,
    stream_aux: &mut GrpcManager<DRSR>,
  ) -> Result<ControlFlow<StatusCode, ()>, E> {
    req.rrd.headers.push_from_iter_many([
      Header::from_name_and_value(
        KnownHeaderName::ContentType.into(),
        [Mime::Grpc.as_str().as_bytes()].into_iter(),
      ),
      Header {
        is_sensitive: false,
        is_trailer: true,
        name: b"grpc-status",
        value: [stream_aux.status_code_mut().number_as_str().as_bytes()].into_iter(),
      },
    ])?;
    Ok(ControlFlow::Continue(()))
  }

  #[inline]
  async fn res(
    &self,
    _: &mut CA,
    _: &mut Self::Aux,
    _: Response<&mut ReqResBuffer>,
    _: &mut GrpcManager<DRSR>,
  ) -> Result<ControlFlow<StatusCode, ()>, E> {
    Ok(ControlFlow::Continue(()))
  }
}
