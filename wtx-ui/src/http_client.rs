use wtx::{
  http::{ClientFramework, ReqBuilder},
  misc::{from_utf8_basic, Uri},
};

pub(crate) async fn http_client(uri: String) -> wtx::Result<()> {
  let client = ClientFramework::tokio_rustls(1).build();
  let res = ReqBuilder::get().send(&client, &Uri::new(uri).to_ref()).await?;
  println!("{}", from_utf8_basic(res.rrd.body()).unwrap());
  Ok(())
}
