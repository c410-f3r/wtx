//! Endpoint with data and parameters

wtx::create_packages_aux_wrapper!();

type Api = ();

#[wtx_macros::pkg(data_format(json), id(super::Api), transport(http))]
mod pkg {
  #[pkg::aux]
  impl super::PkgsAux<(), (), ()> {}

  #[derive(Debug)]
  #[pkg::params]
  pub struct FooParams<'any> {
    bar: &'any str,
  }

  #[derive(Debug)]
  #[pkg::req_data]
  pub(crate) struct FooReq {
    bar: i32
  }

  #[pkg::res_data]
  pub(crate) type FooRes = ();
}

fn main() {
}
