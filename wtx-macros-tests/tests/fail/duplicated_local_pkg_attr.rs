#[wtx::pkg(data_format(json), id(FooId))]
mod pkg {
  #[pkg::aux]
  #[pkg::aux]
  impl Foo {}

  #[derive(Debug)]
  #[pkg::req_data]
  pub struct Req;

  #[pkg::res_data]
  struct Res;
}

#[wtx::pkg(data_format(json), id(FooId))]
mod pkg {
  #[pkg::before_sending]
  #[pkg::before_sending]
  async fn before_sending() -> wtx::Result<()> {
    Ok(())
  }

  #[derive(Debug)]
  #[pkg::req_data]
  pub struct Req;

  #[pkg::res_data]
  struct Res;
}

#[wtx::pkg(data_format(json), id(FooId))]
mod pkg {
  #[derive(Debug)]
  #[pkg::req_data]
  #[pkg::req_data]
  pub struct Req;

  #[pkg::res_data]
  struct Res;
}

fn main() {
}
