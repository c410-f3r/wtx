//! gRPC client that uses the structure definitions found in the `wtx_instances::grpc_bindings`
//! module.

extern crate tokio;
extern crate wtx;
extern crate wtx_examples;

use std::borrow::Cow;
use wtx::{
  codec::format::QuickProtobuf,
  grpc::GrpcClient,
  http::{MsgBuffer, client_pool::ClientPoolBuilder},
};
use wtx_examples::grpc_bindings::wtx::{GenericRequest, GenericResponse};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let msg_buffer =
    MsgBuffer::from_uri("http://127.0.0.1:9000/wtx.GenericService/generic_method".into());
  let pool = ClientPoolBuilder::tokio(1).build();
  let mut client = GrpcClient::new(pool, QuickProtobuf);
  let res = client
    .send_unary_req(
      GenericRequest {
        generic_request_field0: Cow::Borrowed(b"generic_request_value"),
        generic_request_field1: 123,
      },
      msg_buffer,
    )
    .await?;
  let generic_response: GenericResponse =
    client.des_from_res_bytes(&mut res.msg_data.body.as_ref())?;
  println!("{generic_response:?}");
  Ok(())
}
