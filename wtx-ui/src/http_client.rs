use wtx::{
  http::{Client, Method, ReqResBuffer},
  misc::{from_utf8_basic, TokioRustlsConnector},
};

pub(crate) async fn http_client(uri: String) -> wtx::Result<()> {
  let client = Client::tokio(1, |uri| async move {
    TokioRustlsConnector::from_webpki_roots().with_tcp_stream(uri.host(), uri.hostname()).await
  });
  let mut rrb = ReqResBuffer::default();
  rrb.set_uri_from_str(&uri).unwrap();
  let res = client.send(Method::Get, rrb).await.unwrap();
  println!("{}", from_utf8_basic(res.rrd.body()).unwrap());
  Ok(())
}
