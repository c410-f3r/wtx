#[wtx_macros::pkg(api(Foo), data_format(json))]
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
