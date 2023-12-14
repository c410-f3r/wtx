#[wtx_macros::pkg(api(Foo), data_format(json))]
mod pkg {
  #[pkg::req_data]
  struct Req;

  #[pkg::res_data]
  struct Res<T>(T);
}

fn main() {
}
