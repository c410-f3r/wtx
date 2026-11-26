//! gRPC server that uses the structure definitions found in the `wtx_instances::grpc_bindings`
//! module.

extern crate tokio;
extern crate wtx;
extern crate wtx_examples;

use std::borrow::Cow;
use wtx::{
  codec::format::QuickProtobuf,
  grpc::{GrpcManager, GrpcMiddleware},
  http::{
    StatusCode,
    http2_server_framework::{Http2ServerFramework, HttpRouter, State, post},
  },
  tls::TlsConfig,
};
use wtx_examples::{
  PUBLIC_KEY, SECRET_KEY,
  grpc_bindings::wtx::{GenericRequest, GenericResponse},
  host_from_args,
};

fn main() -> wtx::Result<()> {
  Http2ServerFramework::tokio(TlsConfig::from_keys_pem(PUBLIC_KEY, SECRET_KEY)?)?
    .set_data(GrpcManager::from_drsr(QuickProtobuf))
    .set_error_cb(|err| eprintln!("Error: {err}"))
    .set_http_router(HttpRouter::new(
      wtx::paths!(("wtx.GenericService/generic_method", post(wtx_generic_service_generic_method))),
      GrpcMiddleware,
    )?)
    .run_in_threads(&host_from_args())
}

async fn wtx_generic_service_generic_method(
  State { data, req }: State<'_, GrpcManager<QuickProtobuf>>,
) -> wtx::Result<StatusCode> {
  let _generic_request: GenericRequest = data.des_from_req_bytes(&req.msg_data.body)?;
  req.clear();
  data.ser_to_res_bytes(
    &mut req.msg_data.body,
    GenericResponse {
      generic_response_field0: Cow::Borrowed(b"generic_response_value"),
      generic_response_field1: 321,
    },
  )?;
  Ok(StatusCode::Ok)
}
