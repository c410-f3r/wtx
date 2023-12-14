//! Custom transport through `transport(Custom)`.

use wtx::client_api_framework::pkg::PkgsAux;
use wtx::client_api_framework::pkg::Package;
use wtx::client_api_framework::network::TransportGroup;
use wtx::client_api_framework::network::transport::Transport;
use wtx::client_api_framework::network::transport::TransportParams;
use core::ops::Range;

struct CustomTransport;

impl<DRSR> Transport<DRSR> for CustomTransport {
  const GROUP: TransportGroup = TransportGroup::Custom("Custom");
  type Params = CustomTransportParams;

  async fn send<P>(
    &mut self,
    _: &mut P,
    _: &mut PkgsAux<P::Api, DRSR, Self::Params>,
  ) -> Result<(), P::Error>
  where
    P: Package<DRSR, Self::Params>,
  {
    Ok(())
  }

  async fn send_and_retrieve<P>(
    &mut self,
    _: &mut P,
    _: &mut PkgsAux<P::Api, DRSR, Self::Params>,
  ) -> Result<Range<usize>, P::Error>
  where
    P: Package<DRSR, Self::Params>,
  {
    Ok(0..0)
  }
}

struct CustomTransportParams(());

impl TransportParams for CustomTransportParams {
  type ExternalRequestParams = ();
  type ExternalResponseParams = ();

  fn ext_req_params(&self) -> &Self::ExternalRequestParams {
    &self.0
  }

  fn ext_req_params_mut(&mut self) -> &mut Self::ExternalRequestParams {
    &mut self.0
  }

  fn ext_res_params(&self) -> &Self::ExternalResponseParams {
    &self.0
  }

    fn ext_res_params_mut(&mut self) -> &mut Self::ExternalResponseParams {
    &mut self.0
  }

    fn reset(&mut self) {}
}

type Nothing = ();

#[wtx_macros::pkg(api(super::Nothing), data_format(json), transport(custom(crate::CustomTransport)))]
mod pkg {
  #[pkg::req_data]
  struct Req;

  #[pkg::res_data]
  struct Res;
}

fn main() {
}
