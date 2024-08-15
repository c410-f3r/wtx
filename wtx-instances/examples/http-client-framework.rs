//! Http2 CLI framework

use wtx::{
  http::{ClientFramework, Method, ReqResBuffer},
  misc::{from_utf8_basic, Uri},
};

#[tokio::main]
async fn main() {
  let client = ClientFramework::tokio_rustls(1).build();
  let buffer = ReqResBuffer::default();
  let hello = client
    .send(Method::Get, buffer, &Uri::new("https://www.google.com:443/search?q=hello"))
    .await
    .unwrap();
  println!("{}", from_utf8_basic(hello.rrd.body()).unwrap());
  let world = client
    .send(Method::Get, hello.rrd, &Uri::new("https://www.google.com:443/search?q=world"))
    .await
    .unwrap();
  println!("{}", from_utf8_basic(world.rrd.body()).unwrap());
}
