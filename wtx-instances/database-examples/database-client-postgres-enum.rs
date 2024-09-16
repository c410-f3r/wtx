//! ```sql
//! CREATE TYPE custom_enum AS ENUM ('foo', 'bar', 'baz');
//! CREATE TABLE custom_enum_table (id INT, custom_enum CUSTOM_ENUM);
//! ```

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use wtx::database::{
  client::postgres::{DecodeValue, EncodeValue, Postgres},
  Decode, Encode, Executor as _, Record,
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = "postgres://USER:PASSWORD@localhost/DATABASE";
  let mut executor = wtx_instances::executor(&uri).await?;
  let _ = executor
    .execute_with_stmt("INSERT INTO custom_enum_table VALUES ($1, $2)", (1, Enum::Bar))
    .await?;
  let record = executor.fetch_with_stmt("SELECT * FROM custom_enum_table;", ()).await?;
  assert_eq!(record.decode::<_, i32>(0)?, 1);
  assert_eq!(record.decode::<_, Enum>(1)?, Enum::Bar);
  Ok(())
}

#[derive(Debug, PartialEq)]
enum Enum {
  Foo,
  Bar,
  Baz,
}

impl Decode<'_, Postgres<wtx::Error>> for Enum {
  fn decode(input: &DecodeValue<'_>) -> Result<Self, wtx::Error> {
    let s = <&str as Decode<Postgres<wtx::Error>>>::decode(input)?;
    Ok(match s {
      "foo" => Self::Foo,
      "bar" => Self::Bar,
      "baz" => Self::Baz,
      _ => return Err(wtx::Error::UnexpectedString { length: 3 }),
    })
  }
}

impl Encode<Postgres<wtx::Error>> for Enum {
  fn encode(&self, ev: &mut EncodeValue<'_, '_>) -> Result<(), wtx::Error> {
    let s = match self {
      Self::Foo => "foo",
      Self::Bar => "bar",
      Self::Baz => "baz",
    };
    <_ as Encode<Postgres<wtx::Error>>>::encode(&s, ev)?;
    Ok(())
  }
}
