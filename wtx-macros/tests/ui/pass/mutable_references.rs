//! Uses mutable references to send packages

wtx::create_packages_aux_wrapper!();

use wtx::client_api_framework::network::transport::SendingTransport;

type Api = ();

#[wtx_macros::pkg(data_format(json), id(super::Api), transport(stub))]
mod pkg {
  use wtx::client_api_framework::network::transport::TransportParams;

  #[pkg::aux]
  impl<A, DRSR, TP> super::PkgsAux<A, DRSR, TP>
  where
    TP: TransportParams
  {}

  #[derive(Debug)]
  #[pkg::req_data]
  pub(crate) struct FooReq;

  #[pkg::res_data]
  pub(crate) type FooRes = ();
}

fn main() {
  let mut api = ();
  let mut drsr = ();
  let mut tp = ();
  let mut trans = ();
  let mut pkgs_aux = PkgsAux::from_minimum(&mut api, &mut drsr, &mut tp);
  let _ = trans.send(&mut pkgs_aux.foo().build(), &mut pkgs_aux);
}
