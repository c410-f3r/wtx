#[wtx::pkg(data_format(json), id(FooId))]
mod pkg {
  #[derive(Debug)]
  #[pkg::params]
  pub struct Rfewr;

  #[derive(Debug)]
  #[pkg::req_data]
  pub struct Req;

  #[pkg::res_data]
  struct Res;
}

fn main() {
}
