//! gRPC client that uses the structure definitions found in the `wtx_instances::grpc_bindings`
//! module.

extern crate tokio;
extern crate wtx;
extern crate wtx_examples;

use std::borrow::Cow;
use wtx::{
  codec::format::QuickProtobuf,
  grpc::GrpcClient,
  http::{ReqResBuffer, client_pool::ClientPoolBuilder},
  rng::{ChaCha20, CryptoSeedableRng},
  tls::TlsModePlainText,
};
use wtx_examples::grpc_bindings::wtx::{GenericRequest, GenericResponse};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let rrb = ReqResBuffer::empty();
  let rng = ChaCha20::from_std_random()?;
  let uri_ref = rrb.uri.to_ref();
  let pool = ClientPoolBuilder::tokio(1).aux((), |_: &()| TlsModePlainText).build(rng);
  let mut guard = pool.lock(&uri_ref).await?;
  let mut client = GrpcClient::new(&mut guard.client, QuickProtobuf);
  let res = client
    .send_unary_req(
      GenericRequest {
        generic_request_field0: Cow::Borrowed(b"generic_request_value"),
        generic_request_field1: 123,
      },
      rrb,
      "http://127.0.0.1:9000/wtx.GenericService/generic_method".into(),
    )
    .await?;
  let generic_response: GenericResponse = client.des_from_res_bytes(&mut res.rrd.body.as_ref())?;
  println!("{generic_response:?}");
  Ok(())
}
