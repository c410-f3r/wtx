use crate::{
  http::{HttpClient, Method, ReqResBuffer, client_pool::ClientPoolBuilder},
  misc::Uri,
};

#[tokio::test]
async fn popular_sites() {
  let uri = Uri::new("https://github.com");
  let mut client = ClientPoolBuilder::tokio_rustls(1).build();
  let _res = client.send_recv_single(Method::Get, ReqResBuffer::empty(), &uri).await.unwrap();

  let uri = Uri::new("https://duckduckgo.com");
  let mut client = ClientPoolBuilder::tokio_rustls(1).build();
  let _res = client.send_recv_single(Method::Get, ReqResBuffer::empty(), &uri).await.unwrap();

  let uri = Uri::new("https://www.google.com");
  let mut client = ClientPoolBuilder::tokio_rustls(1).build();
  let _res = client.send_recv_single(Method::Get, ReqResBuffer::empty(), &uri).await.unwrap();
}
