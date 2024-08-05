//! Postgres client

use tokio::net::TcpStream;
use wtx::{
  database::{
    client::postgres::{Config, Executor, ExecutorBuffer},
    Executor as _, Record, Records, TransactionManager,
  },
  misc::{TokioRustlsConnector, UriRef},
  rng::StdRng,
};

#[tokio::main]
async fn main() {
  let uri = wtx_instances::uri_from_args();
  let uri_ref = UriRef::new(uri.as_str());
  let config = Config::from_uri(&uri_ref).unwrap();
  let mut rng = StdRng::default();
  let mut exec = Executor::connect_encrypted(
    &config,
    ExecutorBuffer::with_default_params(&mut rng).unwrap(),
    TcpStream::connect(uri_ref.host()).await.unwrap(),
    &mut rng,
    |stream| {
      TokioRustlsConnector::from_webpki_roots()
        .push_certs(include_bytes!("../../.certs/root-ca.crt"))
        .unwrap()
        .with_generic_stream(uri_ref.hostname(), stream)
    },
  )
  .await
  .unwrap();
  let mut tm = exec.transaction().await.unwrap();
  tm.executor()
    .execute("CREATE TABLE IF NOT EXISTS example(id INT, name VARCHAR)", |_| {})
    .await
    .unwrap();
  let _ = tm
    .executor()
    .execute_with_stmt("INSERT INTO foo VALUES ($1, $2), ($3, $4)", (1u32, "one", 2u32, "two"))
    .await
    .unwrap();
  tm.commit().await.unwrap();
  let records = exec
    .fetch_many_with_stmt("SELECT id, name FROM example;", (), |_| Ok::<_, wtx::Error>(()))
    .await
    .unwrap();
  assert_eq!(records.get(0).as_ref().and_then(|record| record.decode("id").ok()), Some(1));
  assert_eq!(records.get(1).as_ref().and_then(|record| record.decode("name").ok()), Some("two"));
}
