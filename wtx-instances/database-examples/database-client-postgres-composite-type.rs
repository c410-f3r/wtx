//! ```sql
//! CREATE TYPE custom_composite_type AS (int_value INT, varchar_value VARCHAR);
//! CREATE TABLE custom_composite_table (id INT, custom_composite_type CUSTOM_COMPOSITE_TYPE);
//! ```

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use wtx::database::{
  client::postgres::{DecodeValue, EncodeValue, Postgres, StructDecoder, StructEncoder},
  Decode, Encode, Executor as _, Record,
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = "postgres://USER:PASSWORD@localhost/DATABASE";
  let mut executor = wtx_instances::executor_postgres(&uri).await?;
  let _ = executor
    .execute_with_stmt(
      "INSERT INTO custom_composite_table VALUES ($1, $2::custom_composite_type)",
      (1, CustomCompositeType(2, 9)),
    )
    .await?;
  let record = executor.fetch_with_stmt("SELECT * FROM custom_composite_table", ()).await?;
  assert_eq!(record.decode::<_, i32>(0)?, 1);
  assert_eq!(record.decode::<_, CustomCompositeType>(1)?, CustomCompositeType(2, 9));
  Ok(())
}

#[derive(Debug, PartialEq)]
struct CustomCompositeType(u32, u64);

impl Decode<'_, Postgres<wtx::Error>> for CustomCompositeType {
  fn decode(input: &DecodeValue<'_>) -> Result<Self, wtx::Error> {
    let mut sd = StructDecoder::<wtx::Error>::new(input);
    Ok(Self(sd.decode()?, sd.decode()?))
  }
}

impl Encode<Postgres<wtx::Error>> for CustomCompositeType {
  fn encode(&self, ev: &mut EncodeValue<'_, '_>) -> Result<(), wtx::Error> {
    let _ev = StructEncoder::<wtx::Error>::new(ev)?.encode(self.0)?.encode(self.1)?;
    Ok(())
  }
}
