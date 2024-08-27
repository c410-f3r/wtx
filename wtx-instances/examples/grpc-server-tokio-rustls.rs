//! gRPC server that uses the structure definitions found in the `wtx_instances::grpc_bindings`
//! module.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use std::borrow::Cow;
use wtx::{
  data_transformation::dnsn::QuickProtobuf,
  grpc::{GrpcStatusCode, Server, ServerData},
  http::{
    server_framework::{post, Router},
    ReqResBuffer, Request, StatusCode,
  },
};
use wtx_instances::grpc_bindings::wtx::{GenericRequest, GenericResponse};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let router = Router::paths(wtx::paths!((
    "wtx.GenericService/generic_method",
    post(wtx_generic_service_generic_method)
  ),));
  Server::new(router)
    .listen_tls(
      (wtx_instances::CERT, wtx_instances::KEY),
      &wtx_instances::host_from_args(),
      |error| eprintln!("{error:?}"),
    )
    .await
}

async fn wtx_generic_service_generic_method(
  req: &mut Request<ReqResBuffer>,
  mut sd: ServerData<QuickProtobuf>,
) -> wtx::Result<(StatusCode, GrpcStatusCode)> {
  let _generic_request: GenericRequest = sd.des_from_req_bytes(&req.rrd.data)?;
  req.rrd.clear();
  sd.ser_to_res_bytes(
    &mut req.rrd.data,
    GenericResponse {
      generic_response_field0: Cow::Borrowed(b"generic_response_value"),
      generic_response_field1: 321,
    },
  )?;
  Ok((StatusCode::Ok, GrpcStatusCode::Ok))
}
