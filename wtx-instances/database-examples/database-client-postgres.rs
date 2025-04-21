//! Demonstrates different interactions with a PostgreSQL database.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use wtx::database::{Executor as _, Record, Records};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = "postgres://USER:PASSWORD@localhost/DATABASE";
  let mut executor = wtx_instances::executor_postgres(uri).await?;
  executor
    .transaction(|this| async {
      this.execute("CREATE TABLE IF NOT EXISTS example(id INT, name VARCHAR)", |_| Ok(())).await?;
      this
        .execute_with_stmt("INSERT INTO foo VALUES ($1, $2), ($3, $4)", (1u32, "one", 2u32, "two"))
        .await?;
      Ok(((), this))
    })
    .await?;
  let records = executor
    .fetch_many_with_stmt("SELECT id, name FROM example", (), |_| Ok::<_, wtx::Error>(()))
    .await?;
  assert_eq!(records.get(0).as_ref().and_then(|record| record.decode("id").ok()), Some(1));
  assert_eq!(records.get(1).as_ref().and_then(|record| record.decode("name").ok()), Some("two"));
  Ok(())
}
