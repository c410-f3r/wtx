use crate::{
  http::{HttpClient, Method, ReqResBuffer, client_pool::ClientPoolBuilder},
  misc::Uri,
};

#[tokio::test]
async fn popular_sites() {
  let uri = Uri::new("https://github.com");
  let _res = ClientPoolBuilder::tokio_rustls(1)
    .build()
    .lock(&uri)
    .await
    .unwrap()
    .client
    .send_recv_single(Method::Get, &uri, ReqResBuffer::empty())
    .await
    .unwrap();

  let uri = Uri::new("https://duckduckgo.com");
  let _res = ClientPoolBuilder::tokio_rustls(1)
    .build()
    .lock(&uri)
    .await
    .unwrap()
    .client
    .send_recv_single(Method::Get, &uri, ReqResBuffer::empty())
    .await
    .unwrap();

  let uri = Uri::new("https://www.google.com");
  let _res = ClientPoolBuilder::tokio_rustls(1)
    .build()
    .lock(&uri)
    .await
    .unwrap()
    .client
    .send_recv_single(Method::Get, &uri, ReqResBuffer::empty())
    .await
    .unwrap();
}
