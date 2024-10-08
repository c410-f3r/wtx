use crate::{
  http::{
    client_framework::{ClientFramework, ReqBuilder},
    ReqResBuffer,
  },
  misc::Uri,
};

#[tokio::test]
async fn popular_sites() {
  let _res = ReqBuilder::get(ReqResBuffer::empty())
    .send(&ClientFramework::tokio_rustls(1).build(), &Uri::new("https://github.com"))
    .await
    .unwrap();
  let _res = ReqBuilder::get(ReqResBuffer::empty())
    .send(&ClientFramework::tokio_rustls(1).build(), &Uri::new("https://duckduckgo.com"))
    .await
    .unwrap();
  let _res = ReqBuilder::get(ReqResBuffer::empty())
    .send(&ClientFramework::tokio_rustls(1).build(), &Uri::new("https://www.google.com"))
    .await
    .unwrap();
}
