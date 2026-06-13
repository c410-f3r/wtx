//! gRPC server that uses the structure definitions found in the `wtx_instances::grpc_bindings`
//! module.

extern crate tokio;
extern crate wtx;
extern crate wtx_examples;

use std::borrow::Cow;
use wtx::{
  codec::format::QuickProtobuf,
  executor::TokioExecutor,
  grpc::{GrpcManager, GrpcMiddleware},
  http::{
    StatusCode,
    http2_server_framework::{Http2ServerFramework, HttpRouter, State, post},
  },
};
use wtx_examples::grpc_bindings::wtx::{GenericRequest, GenericResponse};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let router = HttpRouter::new(
    wtx::paths!(("wtx.GenericService/generic_method", post(wtx_generic_service_generic_method))),
    GrpcMiddleware,
  )?;
  Http2ServerFramework::new(TokioExecutor)?
    .set_data(GrpcManager::from_drsr(QuickProtobuf))
    .run(&wtx_examples::host_from_args(), router)
    .await
}

async fn wtx_generic_service_generic_method(
  state: State<'_, GrpcManager<QuickProtobuf>>,
) -> wtx::Result<StatusCode> {
  let _generic_request: GenericRequest = state.data.des_from_req_bytes(&state.req.msg_data.body)?;
  state.req.clear();
  state.data.ser_to_res_bytes(
    &mut state.req.msg_data.body,
    GenericResponse {
      generic_response_field0: Cow::Borrowed(b"generic_response_value"),
      generic_response_field1: 321,
    },
  )?;
  Ok(StatusCode::Ok)
}
