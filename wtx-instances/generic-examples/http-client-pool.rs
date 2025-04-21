//! Fetches and prints the response body of a provided URI.
//!
//! Currently, only HTTP/2 is supported.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use wtx::{
  http::{HttpClient, Method, ReqResBuffer, client_pool::ClientPoolBuilder},
  misc::{Uri, from_utf8_basic},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("SOME_URI");
  let mut pool = ClientPoolBuilder::tokio(1).build();
  let res = pool.send_recv_single(Method::Get, ReqResBuffer::empty(), &uri).await?;
  println!("{}", from_utf8_basic(&res.rrd.body)?);
  Ok(())
}
