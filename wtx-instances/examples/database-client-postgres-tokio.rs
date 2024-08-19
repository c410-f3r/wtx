//! Demonstrates different interactions with a PostgreSQL database.
//!
//! USAGE: `./program postgres://USER:PASSWORD@localhost:5432/DATABASE`

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use tokio::net::TcpStream;
use wtx::{
  database::{
    client::postgres::{Config, Executor, ExecutorBuffer},
    Executor as _, Record, Records, TransactionManager,
  },
  misc::{NoStdRng, Uri},
};

#[tokio::main]
async fn main() {
  let uri = Uri::new(wtx_instances::uri_from_args());
  let mut rng = NoStdRng::default();
  let mut exec = Executor::connect(
    &Config::from_uri(&uri.to_ref()).unwrap(),
    ExecutorBuffer::with_default_params(&mut rng).unwrap(),
    &mut rng,
    TcpStream::connect(uri.host()).await.unwrap(),
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
