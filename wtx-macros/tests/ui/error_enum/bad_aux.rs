#[wtx_macros::pkg(data_format(json), id(FooId))]
mod pkg {
  #[pkg::aux]
  impl Foo {
    type Foo = i32;
  }

  #[pkg::req_data]
  struct Req(
    i32
  );

  #[pkg::res_data]
  struct Res;
}

fn main() {
}
