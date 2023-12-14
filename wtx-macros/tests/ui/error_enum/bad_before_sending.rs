#[wtx_macros::pkg(api(Foo), data_format(json))]
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
