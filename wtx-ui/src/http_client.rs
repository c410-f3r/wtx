use wtx::{
  http::{Client, Method, ReqResBuffer},
  misc::{from_utf8_basic, Uri},
};

pub(crate) async fn http_client(uri: String) -> wtx::Result<()> {
  let res = Client::tokio_rustls(1)
    .build()
    .send(Method::Get, ReqResBuffer::default(), &Uri::new(uri).to_ref())
    .await
    .unwrap();
  println!("{}", from_utf8_basic(res.rrd.body()).unwrap());
  Ok(())
}
