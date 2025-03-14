//! Demonstrates a MySQL query.
//!
//! This snippet requires ~40 dependencies and has an optimized binary size of ~600K.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use tokio::net::TcpStream;
use wtx::{
  database::{
    Executor, Record, Records,
    client::mysql::{Config, ExecutorBuffer, MysqlExecutor},
  },
  misc::{Uri, Xorshift64},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("mysql://USER:PASSWORD@localhost/DATABASE");
  let mut rng = Xorshift64::from(wtx::misc::simple_seed());
  let mut executor = MysqlExecutor::connect(
    &Config::from_uri(&uri)?,
    ExecutorBuffer::new(usize::MAX, &mut rng),
    TcpStream::connect(uri.hostname_with_implied_port()).await?,
  )
  .await?;
  let records = executor
    .fetch_many_with_stmt("SELECT id, name FROM example", (), |_| Ok::<_, wtx::Error>(()))
    .await?;
  assert_eq!(records.get(0).as_ref().and_then(|record| record.decode("id").ok()), Some(1));
  assert_eq!(records.get(1).as_ref().and_then(|record| record.decode("name").ok()), Some("two"));
  Ok(())
}
