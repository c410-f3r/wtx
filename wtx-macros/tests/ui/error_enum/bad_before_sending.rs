#[wtx_macros::pkg(data_format(json), id(FooId))]
mod pkg {
  #[pkg::before_sending]
  async fn before_sending(foo: i32) -> wtx::Result<()> {
    Ok(())
  }

  #[pkg::req_data]
  struct Req;

  #[pkg::res_data]
  struct Res;
}

fn main() {
}
