use crate::{
  grpc::GrpcManager,
  http::{
    Header, KnownHeaderName, Mime, MsgBufferString, Request, Response, StatusCode,
    http2_server_framework::Middleware,
  },
};
use core::ops::ControlFlow;

/// Applies gRPC headers
#[derive(Debug)]
pub struct GrpcMiddleware;

impl<DRSR, E> Middleware<GrpcManager<DRSR>, E> for GrpcMiddleware
where
  E: From<crate::Error>,
{
  type Aux = ();

  #[inline]
  fn aux(&self) -> Self::Aux {}

  #[inline]
  async fn req(
    &self,
    _: &mut GrpcManager<DRSR>,
    _: &mut Self::Aux,
    _: &mut Request<MsgBufferString>,
  ) -> Result<ControlFlow<StatusCode, ()>, E> {
    Ok(ControlFlow::Continue(()))
  }

  #[inline]
  async fn res(
    &self,
    data: &mut GrpcManager<DRSR>,
    _: &mut Self::Aux,
    res: Response<&mut MsgBufferString>,
  ) -> Result<ControlFlow<StatusCode, ()>, E> {
    res.msg_data.headers.push_from_iter_many([
      Header::from_name_and_value(
        KnownHeaderName::ContentType.into(),
        [Mime::ApplicationGrpc.as_str()].into_iter(),
      ),
      Header::new(false, true, "grpc-status", [data.status_code_mut().as_str()].into_iter()),
    ])?;
    Ok(ControlFlow::Continue(()))
  }
}
