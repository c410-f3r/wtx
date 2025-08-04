#[wtx_macros::pkg(data_format(json), id(FooId), transport(dasdasd))]
mod pkg {
  #[pkg::req_data]
  struct Req;

  #[pkg::res_data]
  struct Res;
}

fn main() {
}
