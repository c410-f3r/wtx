//! Any request with lifetimes and custom data methods must have lifetimes that match
//! both request and method signatures.

wtx::create_packages_aux_wrapper!();

type Api = ();

#[wtx_macros::pkg(data_format(json), id(super::Api), transport(http))]
mod pkg {
  #[pkg::aux]
  impl super::PkgsAux<(), (), ()> {
    #[pkg::aux_data]
    fn foo_data<'any>(&mut self, param: &'any ()) -> wtx::Result<FooReq<'any>> {
      Ok(param)
    }
  }

  #[pkg::req_data]
  pub(crate) type FooReq<'any> = &'any ();

  #[pkg::res_data]
  pub(crate) type FooRes = ();
}

fn main() {
}
