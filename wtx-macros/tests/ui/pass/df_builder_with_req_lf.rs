//! Any request with lifetimes and custom data methods must have lifetimes that match
//! both request and method signatures.

wtx::client_api_framework::create_packages_aux_wrapper!();

type Api = ();

#[wtx_macros::pkg(api(super::Api), data_format(json), transport(http))]
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
