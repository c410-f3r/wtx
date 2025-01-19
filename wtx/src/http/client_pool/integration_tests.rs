use crate::{
  http::{client_pool::ClientPoolBuilder, ReqBuilder, ReqResBuffer},
  misc::Uri,
};

#[tokio::test]
async fn popular_sites() {
  let uri = Uri::new("https://github.com");
  let _res = ReqBuilder::get(ReqResBuffer::empty())
    .send(&mut ClientPoolBuilder::tokio_rustls(1).build().lock(&uri).await.unwrap().client, &uri)
    .await
    .unwrap();

  let uri = Uri::new("https://duckduckgo.com");
  let _res = ReqBuilder::get(ReqResBuffer::empty())
    .send(&mut ClientPoolBuilder::tokio_rustls(1).build().lock(&uri).await.unwrap().client, &uri)
    .await
    .unwrap();

  let uri = Uri::new("https://www.google.com");
  let _res = ReqBuilder::get(ReqResBuffer::empty())
    .send(&mut ClientPoolBuilder::tokio_rustls(1).build().lock(&uri).await.unwrap().client, &uri)
    .await
    .unwrap();
}
