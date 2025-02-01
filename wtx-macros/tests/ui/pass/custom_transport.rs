//! Custom transport through `transport(Custom)`.

use core::ops::Range;
use wtx::client_api_framework::{
  network::{
    transport::{RecievingTransport, SendingTransport, Transport, TransportParams},
    TransportGroup,
  },
  pkg::{Package, PkgsAux},
  Api,
};

struct CustomTransport;

impl RecievingTransport<CustomTransportParams> for CustomTransport {
  #[inline]
  async fn recv<A, DRSR>(
    &mut self,
    _: &mut PkgsAux<A, DRSR, CustomTransportParams>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
  {
    Ok(0..0)
  }
}

impl SendingTransport<CustomTransportParams> for CustomTransport {
  #[inline]
  async fn send<A, DRSR, P>(
    &mut self,
    _: &mut P,
    _: &mut PkgsAux<A, DRSR, CustomTransportParams>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: Package<A, DRSR, Self, CustomTransportParams>,
  {
    Ok(())
  }
}

impl Transport<CustomTransportParams> for CustomTransport {
  const GROUP: TransportGroup = TransportGroup::Custom("Custom");
  type Inner = Self;
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

#[wtx_macros::pkg(data_format(json), id(super::Nothing), transport(custom(crate::CustomTransportParams)))]
mod pkg {
  #[pkg::req_data]
  struct Req;

  #[pkg::res_data]
  struct Res;
}

fn main() {}