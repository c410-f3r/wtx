#[wtx_macros::pkg(data_format(json), id(FooId))]
mod pkg {
  #[derive(Debug)]
  #[pkg::req_data]
  pub struct Req(#[pkg::bar] i32);

  #[pkg::res_data]
  struct Res;
}

#[wtx_macros::pkg(data_format(json), id(FooId))]
mod pkg {
  #[derive(Debug)]
  #[pkg::req_data]
  pub struct Req(#[pkg::field(foo = "1111")] i32);

  #[pkg::res_data]
  struct Res;
}

fn main() {
}
