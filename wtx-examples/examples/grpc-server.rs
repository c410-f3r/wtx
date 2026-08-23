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
  rng::{ChaCha20, CryptoSeedableRng},
  tls::TlsConfig,
};
use wtx_examples::{
  PUBLIC_KEY, SECRET_KEY,
  grpc_bindings::wtx::{GenericRequest, GenericResponse},
  host_from_args,
};

fn main() -> wtx::Result<()> {
  let mut rng = ChaCha20::from_std_random()?;
  let tls_config = TlsConfig::from_keys_pem(PUBLIC_KEY.try_into()?, &mut rng, SECRET_KEY)?;
  let router = HttpRouter::new(
    wtx::paths!(("wtx.GenericService/generic_method", post(wtx_generic_service_generic_method))),
    GrpcMiddleware,
  )?;
  Http2ServerFramework::new(TokioExecutor::default(), rng, tls_config)?
    .set_data(GrpcManager::from_drsr(QuickProtobuf))
    .set_error_cb(|err| eprintln!("Error: {err}"))
    .run_in_threads(&host_from_args(), router)
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
