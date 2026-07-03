//! gRPC client that uses the structure definitions found in the `wtx_instances::grpc_bindings`
//! module.

extern crate tokio;
extern crate wtx;
extern crate wtx_examples;

use std::borrow::Cow;
use wtx::{
  codec::format::QuickProtobuf,
  executor::TokioExecutor,
  grpc::GrpcClient,
  http::{MsgBufferStr, http2_client_pool::Http2ClientPoolBuilder},
  rng::{ChaCha20, CryptoSeedableRng as _},
  tls::{TlsConfig, TlsModeVerified},
};
use wtx_examples::{
  ROOT_CA,
  grpc_bindings::wtx::{GenericRequest, GenericResponse},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = "https://127.0.0.1:9000/wtx.GenericService/generic_method";
  let mut client = GrpcClient::new(
    Http2ClientPoolBuilder::new(
      TokioExecutor::default(),
      1,
      ChaCha20::from_getrandom()?,
      TlsConfig::from_trust_anchors_pem(TlsModeVerified::default(), [ROOT_CA])?.into(),
    )?
    .build(),
    QuickProtobuf,
  );
  let res = client
    .send_unary_req(
      GenericRequest {
        generic_request_field0: Cow::Borrowed(b"generic_request_value"),
        generic_request_field1: 123,
      },
      MsgBufferStr::from_uri(uri.into()),
    )
    .await?;
  let generic_response: GenericResponse = client.des_from_res_bytes(&res.msg_data.body)?;
  println!("{generic_response:?}");
  Ok(())
}
