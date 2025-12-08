use crate::{
  http::{HttpClient, ReqBuilder, ReqResBuffer, client_pool::ClientPoolBuilder},
  misc::UriRef,
};

#[tokio::test]
async fn popular_sites() {
  send_recv("https://github.com".into()).await;
  send_recv("https://duckduckgo.com".into()).await;
  send_recv("https://www.google.com".into()).await;
}

async fn send_recv(uri: UriRef<'_>) {
  let client = ClientPoolBuilder::tokio_rustls(1).build();
  let _res = client.send_req_recv_res(ReqResBuffer::empty(), ReqBuilder::get(uri)).await.unwrap();
}
