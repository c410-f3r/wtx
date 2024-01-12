//! Postgres client

#[path = "./common/mod.rs"]
mod common;

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
  let uri = common::_uri_from_args();
  let uri = UriRef::new(uri.as_str());
  let mut rng = StdRng::default();
  let config = Config::from_uri(&uri).unwrap();
  let eb = ExecutorBuffer::with_default_params(&mut rng);
  let initial_stream = TcpStream::connect(uri.host()).await.unwrap();
  let mut exec =
    Executor::connect_encrypted(&config, eb, initial_stream, &mut rng, |stream| async {
      TokioRustlsConnector::from_webpki_roots()
        .push_certs(include_bytes!("../../.certs/root-ca.crt"))
        .unwrap()
        .with_generic_stream(uri.hostname(), stream)
        .await
    })
    .await
    .unwrap();
  let mut tm = exec.transaction().await.unwrap();
  let _ = tm
    .executor()
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
