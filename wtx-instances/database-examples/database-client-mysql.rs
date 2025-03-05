//! Demonstrates a MySQL query.
//!
//! This snippet requires ~40 dependencies and has an optimized binary size of ~600K.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use wtx::database::{Executor as _, Record, Records};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = "mysql://USER:PASSWORD@localhost/DATABASE";
  let mut executor = wtx_instances::executor_mysql(&uri).await?;
  let records = executor
    .fetch_many_with_stmt("SELECT id, name FROM example", (), |_| Ok::<_, wtx::Error>(()))
    .await?;
  assert_eq!(records.get(0).as_ref().and_then(|record| record.decode("id").ok()), Some(1));
  assert_eq!(records.get(1).as_ref().and_then(|record| record.decode("name").ok()), Some("two"));
  Ok(())
}
