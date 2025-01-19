//! Fetches and prints the response body of a provided URI.
//!
//! This snippet requires ~25 dependencies and has an optimized binary size of ~700K.
//!
//! Currently, only HTTP/2 is supported.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use wtx::{
  http::{client_pool::ClientPoolBuilder, ReqBuilder, ReqResBuffer},
  misc::{from_utf8_basic, Uri},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("http://www.example.com");
  let res = ReqBuilder::get(ReqResBuffer::empty())
    .send(&mut ClientPoolBuilder::tokio(1).build().lock(&uri).await?.client, &uri)
    .await?;
  println!("{}", from_utf8_basic(&res.rrd.body)?);
  Ok(())
}
