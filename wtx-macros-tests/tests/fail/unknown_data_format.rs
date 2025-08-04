#[wtx_macros::pkg(data_format(fdsdfs), id(Foo))]
mod pkg {
  #[pkg::req_data]
  struct Req;

  #[pkg::res_data]
  struct Res;
}

fn main() {
}
