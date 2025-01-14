#[wtx_macros::pkg(data_format(json), id(FooId))]
mod pkg {
  #[pkg::req_data]
  struct Req;

  #[pkg::res_data]
  struct Res<T>(T);
}

fn main() {
}
