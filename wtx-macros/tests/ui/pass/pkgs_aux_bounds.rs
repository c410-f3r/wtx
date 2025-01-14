//! `#[pkg::aux]` accepts bounds

wtx::create_packages_aux_wrapper!();

type Api = ();

/// Generic API
pub trait GenericApi {}

impl GenericApi for &mut Api {}
impl GenericApi for Api {}
impl GenericApi for Box<Api> {}

#[wtx_macros::pkg(data_format(json), id(super::Api), transport(http))]
mod pkg {
  #[pkg::aux]
  impl<A> super::PkgsAux<A, (), ()>
  where
    A: super::GenericApi
  {}

  #[pkg::req_data]
  pub(crate) type FooReq<'any> = &'any ();

  #[pkg::res_data]
  pub(crate) type FooRes = ();
}

fn main() {
}
