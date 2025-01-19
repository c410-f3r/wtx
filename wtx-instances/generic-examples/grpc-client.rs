//! gRPC client that uses the structure definitions found in the `wtx_instances::grpc_bindings`
//! module.
//!
//! This snippet requires ~40 dependencies and has an optimized binary size of ~700K.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use std::borrow::Cow;
use wtx::{
  data_transformation::dnsn::QuickProtobuf,
  grpc::Client,
  http::{client_pool::ClientPoolBuilder, ReqResBuffer, ReqResData},
};
use wtx_instances::grpc_bindings::wtx::{GenericRequest, GenericResponse};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let mut rrb = ReqResBuffer::empty();
  rrb.uri.reset(|el| {
    el.push_str("http://127.0.0.1:9000");
    Ok(())
  })?;
  let uri_ref = rrb.uri.to_ref();
  let pool = ClientPoolBuilder::tokio(1).build();
  let mut guard = pool.lock(&uri_ref).await?;
  let mut client = Client::new(&mut guard.client, QuickProtobuf);
  let res = client
    .send_unary_req(
      ("wtx", "GenericService", "generic_method"),
      GenericRequest {
        generic_request_field0: Cow::Borrowed(b"generic_request_value"),
        generic_request_field1: 123,
      },
      rrb,
    )
    .await?;
  let generic_response: GenericResponse = client.des_from_res_bytes(res.rrd.body())?;
  println!("{:?}", generic_response);
  Ok(())
}
