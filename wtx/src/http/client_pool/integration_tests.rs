use crate::{
  executor::Runtime,
  http::{HttpClient, ReqBuilder, ReqResBuffer, client_pool::ClientPoolBuilder},
  misc::UriRef,
  rng::ChaCha20,
  tls::TlsModeVerifyFull,
};

#[test]
fn popular_sites() {
  Runtime::new().block_on(async move {
    send_recv("https://github.com".into()).await;
    send_recv("https://duckduckgo.com".into()).await;
    send_recv("https://www.google.com".into()).await;
  });
}

async fn send_recv(uri: UriRef<'_>) {
  let client = ClientPoolBuilder::tokio(1)
    .aux((), |_: &()| TlsModeVerifyFull)
    .build(ChaCha20::from_key([1; 32]));
  let _res = client.send_req_recv_res(ReqResBuffer::empty(), ReqBuilder::get(uri)).await.unwrap();
}
