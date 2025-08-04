use wtx::database::client::postgres::Postgres;

#[derive(wtx_macros::FromRecords)]
#[from_records(Postgres<wtx::Error>)]
pub struct Foo {
  pub bar: i32,
  pub baz: i64,
}

fn main() {}
