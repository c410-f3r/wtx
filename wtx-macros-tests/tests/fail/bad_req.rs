#[wtx::pkg(data_format(json), id(FooId))]
mod pkg {
  #[derive(Debug)]
  #[pkg::req_data]
  pub struct Rfdsqw;

  #[pkg::res_data]
  struct Res;
}

fn main() {
}
