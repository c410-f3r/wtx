#[wtx_macros::pkg(api(Foo), data_format(json))]
mod pkg {
  #[derive(Debug)]
  #[pkg::req_data]
  pub struct Req(#[pkg::bar] i32);

  #[pkg::res_data]
  struct Res;
}

#[wtx_macros::pkg(api(Foo), data_format(json))]
mod pkg {
  #[derive(Debug)]
  #[pkg::req_data]
  pub struct Req(#[pkg::field(foo = "1111")] i32);

  #[pkg::res_data]
  struct Res;
}

fn main() {
}
