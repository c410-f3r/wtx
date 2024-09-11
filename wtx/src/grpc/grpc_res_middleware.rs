use crate::{
  grpc::GrpcManager,
  http::{server_framework::ResMiddleware, Header, KnownHeaderName, Mime, ReqResDataMut, Response},
};

/// Applies gRPC headers
#[derive(Debug)]
pub struct GrpcResMiddleware;

impl<CA, DRSR, E, RRD> ResMiddleware<CA, E, GrpcManager<DRSR>, RRD> for GrpcResMiddleware
where
  E: From<crate::Error>,
  RRD: ReqResDataMut,
{
  #[inline]
  async fn apply_res_middleware(
    &self,
    _: &mut CA,
    ra: &mut GrpcManager<DRSR>,
    res: Response<&mut RRD>,
  ) -> Result<(), E> {
    res.rrd.headers_mut().push_from_iter(Header::from_name_and_value(
      KnownHeaderName::ContentType.into(),
      [Mime::Grpc.as_str().as_bytes()],
    ))?;
    res.rrd.headers_mut().push_from_iter(Header {
      is_sensitive: false,
      is_trailer: true,
      name: b"grpc-status",
      value: [ra.status_code_mut().number_as_str().as_bytes()],
    })?;
    Ok(())
  }
}
