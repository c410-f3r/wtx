//! Fetches and prints the response body of a provided URI.
//!
//! Currently, only HTTP/2 is supported.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use wtx::{
  http::{HttpClient, ReqBuilder, ReqResBuffer, client_pool::ClientPoolBuilder},
  misc::{Uri, from_utf8_basic},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("SOME_URI");
  let pool = ClientPoolBuilder::tokio(1).build();
  let res = pool.send_req_recv_res(ReqResBuffer::empty(), ReqBuilder::get(uri.to_ref())).await?;
  println!("{}", from_utf8_basic(&res.rrd.body)?);
  Ok(())
}
