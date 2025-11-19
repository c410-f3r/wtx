//! Sends multiple commands at once and awaits them.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use wtx::{
  collection::ArrayVectorU8,
  database::{Record, Records},
};

const COMMANDS: &[&str] = &["SELECT 0 = $1", "SELECT 1 = $1", "SELECT 2 = $1"];

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = "postgres://USER:PASSWORD@localhost/DATABASE";
  let mut executor = wtx_instances::executor_postgres(uri).await?;
  let mut batch = executor.batch();
  let mut idx: u32 = 0;
  let mut records = ArrayVectorU8::<_, { COMMANDS.len() }>::new();
  for cmd in COMMANDS {
    batch.stmt(cmd, (idx,)).unwrap();
    idx = idx.wrapping_add(1);
  }
  batch.flush(&mut records, |_| Ok(())).await.unwrap();
  for record in records {
    assert_eq!(record.get(0).unwrap().decode::<_, bool>(0).unwrap(), true);
  }
  Ok(())
}
