use wtx::database::client::postgres::Postgres;

#[derive(wtx::FromRecords)]
#[from_records(Postgres<E>, bound = "E: Default", modifier = foo::modifier)]
pub struct Foo {
  pub a: i32,
  pub b: i64,
  #[from_records(ignore)]
  pub c: i32
}

mod foo {
  pub(crate) fn modifier(foo: &mut super::Foo) {
    foo.a = foo.a.wrapping_add(1);
  }
}

fn main() {}
