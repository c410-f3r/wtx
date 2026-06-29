use crate::{
  collections::Vector,
  executor::{StdExecutor, StdRuntime},
  http::{HttpClient, ReqBuilder, http2_client_pool::Http2ClientPoolBuilder},
  misc::UriRef,
  tls::{TlsConfig, TlsModeVerified},
};

#[ignore]
#[test]
fn popular_sites() {
  StdRuntime::new().block_on(async move {
    send_recv("https://github.com".into()).await;
    send_recv("https://duckduckgo.com".into()).await;
    send_recv("https://www.google.com".into()).await;
  });
}

async fn send_recv(uri: UriRef<'_>) {
  let client = Http2ClientPoolBuilder::new(
    StdExecutor::default(),
    1,
    TlsConfig::from_ccadb(TlsModeVerified::default()).unwrap().into(),
  )
  .unwrap()
  .build();
  let _res = client
    .send_req_recv_res(&mut Vector::new(), ReqBuilder::get(uri).into_request())
    .await
    .unwrap();
}
