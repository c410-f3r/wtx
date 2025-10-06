use crate::{
  http::{HttpClient, Method, ReqResBuffer, client_pool::ClientPoolBuilder},
  misc::UriRef,
};

#[tokio::test]
async fn popular_sites() {
  send_recv(&"https://github.com".into()).await;
  send_recv(&"https://duckduckgo.com".into()).await;
  send_recv(&"https://www.google.com".into()).await;
}

async fn send_recv(uri: &UriRef<'_>) {
  let client = ClientPoolBuilder::tokio_rustls(1).build();
  let _res = client.send_recv_single(Method::Get, ReqResBuffer::empty(), uri).await.unwrap();
}
