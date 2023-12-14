#[wtx_macros::pkg(api(Foo), data_format(json))]
mod pkg {
  #[derive(Debug)]
  #[pkg::req_data]
  pub struct Rfdsqw;

  #[pkg::res_data]
  struct Res;
}

fn main() {
}
