#[wtx_macros::pkg(data_format(json_rpc(Bar)), id(Foo))]
mod pkg {
  #[pkg::req_data]
  struct Req;

  #[pkg::res_data]
  struct Res;
}

fn main() {
}
