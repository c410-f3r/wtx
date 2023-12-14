#[wtx_macros::pkg(api(Foo), data_format(json_rpc(Bar)))]
mod pkg {
  #[pkg::req_data]
  struct Req;

  #[pkg::res_data]
  struct Res;
}

fn main() {
}
