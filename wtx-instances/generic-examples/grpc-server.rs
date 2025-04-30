//! gRPC server that uses the structure definitions found in the `wtx_instances::grpc_bindings`
//! module.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use std::borrow::Cow;
use wtx::{
  data_transformation::dnsn::QuickProtobuf,
  grpc::{GrpcManager, GrpcMiddleware},
  http::{
    ReqResBuffer, StatusCode,
    server_framework::{Router, ServerFrameworkBuilder, State, post},
  },
  rng::{Xorshift64, simple_seed},
};
use wtx_instances::grpc_bindings::wtx::{GenericRequest, GenericResponse};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let router = Router::new(
    wtx::paths!(("wtx.GenericService/generic_method", post(wtx_generic_service_generic_method))),
    GrpcMiddleware,
  )?;
  ServerFrameworkBuilder::new(Xorshift64::from(simple_seed()), router)
    .with_stream_aux(|_| QuickProtobuf)
    .tokio_rustls(
      (wtx_instances::CERT, wtx_instances::KEY),
      &wtx_instances::host_from_args(),
      |error| eprintln!("{error}"),
      |_| Ok(()),
      |error| eprintln!("{error}"),
    )
    .await
}

async fn wtx_generic_service_generic_method(
  state: State<'_, (), GrpcManager<QuickProtobuf>, ReqResBuffer>,
) -> wtx::Result<StatusCode> {
  let _generic_request: GenericRequest =
    state.stream_aux.des_from_req_bytes(&mut state.req.rrd.body.as_ref())?;
  state.req.rrd.clear();
  state.stream_aux.ser_to_res_bytes(
    &mut state.req.rrd.body,
    GenericResponse {
      generic_response_field0: Cow::Borrowed(b"generic_response_value"),
      generic_response_field1: 321,
    },
  )?;
  Ok(StatusCode::Ok)
}
