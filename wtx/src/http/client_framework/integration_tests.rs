use crate::{
  http::{ClientFramework, ReqBuilder},
  misc::Uri,
};

#[tokio::test]
async fn popular_sites() {
  let _res = ReqBuilder::get()
    .send(&ClientFramework::tokio_rustls(1).build(), &Uri::new("https://github.com:443"))
    .await
    .unwrap();
  let _res = ReqBuilder::get()
    .send(&ClientFramework::tokio_rustls(1).build(), &Uri::new("https://duckduckgo.com:443"))
    .await
    .unwrap();
  let _res = ReqBuilder::get()
    .send(&ClientFramework::tokio_rustls(1).build(), &Uri::new("https://www.google.com:443"))
    .await
    .unwrap();
}
