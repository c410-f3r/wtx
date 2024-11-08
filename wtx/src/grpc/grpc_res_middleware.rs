use core::ops::ControlFlow;

use crate::{
  grpc::GrpcManager,
  http::{
    server_framework::Middleware, Header, KnownHeaderName, Mime, ReqResBuffer, ReqResDataMut,
    Request, Response, StatusCode,
  },
};

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
    res: &mut Request<ReqResBuffer>,
    sa: &mut GrpcManager<DRSR>,
  ) -> Result<ControlFlow<StatusCode, ()>, E> {
    res.rrd.headers_mut().push_from_iter_many([
      Header::from_name_and_value(
        KnownHeaderName::ContentType.into(),
        [Mime::Grpc.as_str().as_bytes()].into_iter(),
      ),
      Header {
        is_sensitive: false,
        is_trailer: true,
        name: b"grpc-status",
        value: [sa.status_code_mut().number_as_str().as_bytes()].into_iter(),
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
