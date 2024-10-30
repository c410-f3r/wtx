use crate::{
  grpc::GrpcManager,
  http::{
    server_framework::ResMiddleware, Header, KnownHeaderName, Mime, ReqResBuffer, ReqResDataMut,
    Response,
  },
};

/// Applies gRPC headers
#[derive(Debug)]
pub struct GrpcResMiddleware;

impl<CA, DRSR, E> ResMiddleware<CA, E, GrpcManager<DRSR>> for GrpcResMiddleware
where
  E: From<crate::Error>,
{
  #[inline]
  async fn apply_res_middleware(
    &self,
    _: &mut CA,
    res: Response<&mut ReqResBuffer>,
    sa: &mut GrpcManager<DRSR>,
  ) -> Result<(), E> {
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
    Ok(())
  }
}
