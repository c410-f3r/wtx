#[wtx_macros::pkg(api(Foo), data_format(json), transport(dasdasd))]
mod pkg {
  #[pkg::req_data]
  struct Req;

  #[pkg::res_data]
  struct Res;
}

fn main() {
}
