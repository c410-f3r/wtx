//! ```sql
//! CREATE TYPE custom_enum AS ENUM ('foo', 'bar', 'baz');
//! CREATE TABLE custom_enum_table (id INT, custom_enum CUSTOM_ENUM);
//! ```

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use wtx::{
  database::{
    Executor as _, Record, Typed,
    client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, Ty},
  },
  de::{Decode, Encode},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = "postgres://USER:PASSWORD@localhost/DATABASE";
  let mut executor = wtx_instances::executor_postgres(uri).await?;
  executor
    .execute_stmt_none("INSERT INTO custom_enum_table VALUES ($1, $2)", (1, Enum::Bar))
    .await?;
  let record = executor.execute_stmt_single("SELECT * FROM custom_enum_table;", ()).await?;
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
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'_, '_>) -> Result<Self, wtx::Error> {
    let s = <&str as Decode<Postgres<wtx::Error>>>::decode(dw)?;
    Ok(match s {
      "foo" => Self::Foo,
      "bar" => Self::Bar,
      "baz" => Self::Baz,
      _ => return Err(wtx::Error::UnexpectedString { length: 3 }),
    })
  }
}

impl Encode<Postgres<wtx::Error>> for Enum {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), wtx::Error> {
    let s = match self {
      Self::Foo => "foo",
      Self::Bar => "bar",
      Self::Baz => "baz",
    };
    <_ as Encode<Postgres<wtx::Error>>>::encode(&s, ew)?;
    Ok(())
  }
}

impl Typed<Postgres<wtx::Error>> for Enum {
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    None
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    None
  }
}
