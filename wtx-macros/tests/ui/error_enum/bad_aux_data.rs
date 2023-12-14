#[wtx_macros::pkg(api(Foo), data_format(json))]
mod pkg {
  #[pkg::aux]
  impl Foo {
    #[pkg::aux_data]
    fn fdsfqw() {}
  }

  #[pkg::req_data]
  struct FooReq(
    i32
  );

  #[pkg::res_data]
  struct FooRes;
}

fn main() {
}
