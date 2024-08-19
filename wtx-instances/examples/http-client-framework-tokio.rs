//! Fetches and prints the response body of a provided URI.
//!
//! Currently, only HTTP/2 is supported.
//!
//! USAGE: `./program http://www.example.com:80`

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use wtx::{
  http::{ClientFramework, Method, ReqResBuffer},
  misc::{from_utf8_basic, Uri},
};

#[tokio::main]
async fn main() {
  let buffer = ReqResBuffer::default();
  let client = ClientFramework::tokio(1).build();
  let uri = Uri::new(wtx_instances::uri_from_args());
  let res = client.send(Method::Get, buffer, &uri.to_ref()).await.unwrap();
  println!("{}", from_utf8_basic(&res.rrd.data).unwrap());
}
