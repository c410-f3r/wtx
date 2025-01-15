#[wtx_macros::pkg(data_format(json), id(FooId))]
mod pkg {
  #[derive(Debug)]
  #[pkg::req_data]
  pub struct Req;

  #[pkg::req_data]
  struct Res;
}

fn main() {
}
