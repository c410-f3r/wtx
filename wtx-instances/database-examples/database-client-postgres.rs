//! Demonstrates different interactions with a PostgreSQL database.
//!
//! This snippet requires ~40 dependencies and has an optimized binary size of ~600K.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use wtx::database::{Executor as _, Record, Records, TransactionManager};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = "postgres://USER:PASSWORD@localhost/DATABASE";
  let mut executor = wtx_instances::executor(&uri).await?;
  let mut tm = executor.transaction().await?;
  tm.executor().execute("CREATE TABLE IF NOT EXISTS example(id INT, name VARCHAR)", |_| {}).await?;
  let _ = tm
    .executor()
    .execute_with_stmt("INSERT INTO foo VALUES ($1, $2), ($3, $4)", (1u32, "one", 2u32, "two"))
    .await?;
  tm.commit().await?;
  let records = executor
    .fetch_many_with_stmt("SELECT id, name FROM example;", (), |_| Ok::<_, wtx::Error>(()))
    .await?;
  assert_eq!(records.get(0).as_ref().and_then(|record| record.decode("id").ok()), Some(1));
  assert_eq!(records.get(1).as_ref().and_then(|record| record.decode("name").ok()), Some("two"));
  Ok(())
}
