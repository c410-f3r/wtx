//! Demonstrates different interactions with a PostgreSQL database.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use wtx::{
  database::{Executor as _, Record, Records},
  misc::into_rslt,
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = "postgres://USER:PASSWORD@localhost/DATABASE";
  let mut executor = wtx_instances::executor_postgres(uri).await?;
  executor
    .transaction(|this| async {
      this.execute_ignored("CREATE TABLE IF NOT EXISTS example(id INT, name VARCHAR)").await?;
      this
        .execute_stmt_none("INSERT INTO foo VALUES ($1, $2), ($3, $4)", (1u32, "one", 2u32, "two"))
        .await?;
      Ok(((), this))
    })
    .await?;
  let records = executor
    .execute_stmt_many("SELECT id, name FROM example", (), |_| Ok::<_, wtx::Error>(()))
    .await?;
  let record0 = into_rslt(records.get(0))?;
  let record1 = into_rslt(records.get(1))?;
  assert_eq!((record0.decode::<_, u32>(0)?, record0.decode::<_, &str>("name")?), (1, "one"));
  assert_eq!((record1.decode::<_, u32>("id")?, record1.decode::<_, &str>(1)?), (2, "two"));
  Ok(())
}
