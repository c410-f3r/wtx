//! Demonstrates a MySQL query.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use wtx::database::{Executor, Record, Records};
use wtx_instances::executor_mysql;

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let mut executor = executor_mysql("mysql://USER:PASSWORD@localhost/DATABASE").await?;
  let records = executor
    .fetch_many_with_stmt("SELECT id, name FROM example", (), |_| Ok::<_, wtx::Error>(()))
    .await?;
  assert_eq!(records.get(0).as_ref().and_then(|record| record.decode("id").ok()), Some(1));
  assert_eq!(records.get(1).as_ref().and_then(|record| record.decode("name").ok()), Some("two"));
  Ok(())
}
