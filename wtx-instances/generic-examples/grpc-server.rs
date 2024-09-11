//! gRPC server that uses the structure definitions found in the `wtx_instances::grpc_bindings`
//! module.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use std::borrow::Cow;
use wtx::{
  data_transformation::dnsn::QuickProtobuf,
  grpc::{GrpcManager, GrpcResMiddleware},
  http::{
    server_framework::{post, Router, ServerFrameworkBuilder, State},
    ReqResBuffer, StatusCode,
  },
};
use wtx_instances::grpc_bindings::wtx::{GenericRequest, GenericResponse};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let router = Router::new(
    wtx::paths!(("wtx.GenericService/generic_method", post(wtx_generic_service_generic_method))),
    (),
    GrpcResMiddleware,
  )?;
  ServerFrameworkBuilder::new(router)
    .with_req_aux(|| QuickProtobuf::default())
    .listen_tls(
      (wtx_instances::CERT, wtx_instances::KEY),
      &wtx_instances::host_from_args(),
      |error| eprintln!("{error}"),
    )
    .await
}

async fn wtx_generic_service_generic_method(
  state: State<'_, (), GrpcManager<QuickProtobuf>, ReqResBuffer>,
) -> wtx::Result<StatusCode> {
  let _generic_request: GenericRequest = state.ra.des_from_req_bytes(&state.req.rrd.data)?;
  state.req.rrd.clear();
  state.ra.ser_to_res_bytes(
    &mut state.req.rrd.data,
    GenericResponse {
      generic_response_field0: Cow::Borrowed(b"generic_response_value"),
      generic_response_field1: 321,
    },
  )?;
  Ok(StatusCode::Ok)
}
