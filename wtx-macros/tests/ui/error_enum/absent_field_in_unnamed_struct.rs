#[wtx_macros::pkg(data_format(json), id(FooId))]
mod pkg {
  #[pkg::aux]
  impl Foo {}

  #[pkg::req_data]
  struct Req(
    i32
  );

  #[pkg::res_data]
  struct Res;
}

fn main() {
}
