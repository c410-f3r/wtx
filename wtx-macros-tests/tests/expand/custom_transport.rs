//! Custom transport through `transport(Custom)`.

use wtx::client_api_framework::{
  network::{
    transport::{ReceivingTransport, SendingTransport, Transport, TransportParams},
    TransportGroup,
  },
  pkg::{Package, PkgsAux},
  Api, SendBytesSource
};

struct CustomTransport;

impl ReceivingTransport<CustomTransportParams> for CustomTransport {
  #[inline]
  async fn recv<A, DRSR>(
    &mut self,
    _: &mut PkgsAux<A, DRSR, CustomTransportParams>,
    _: Self::ReqId,
  ) -> Result<(), A::Error>
  where
    A: Api,
  {
    Ok(())
  }
}

impl SendingTransport<CustomTransportParams> for CustomTransport {
  async fn send_bytes<A, DRSR>(
    &mut self,
    _: SendBytesSource<'_>,
    _: &mut PkgsAux<A, DRSR, CustomTransportParams>,
  ) -> Result<(), A::Error>
  where
    A: Api
  {
    Ok(())
  }

  async fn send_pkg<A, DRSR, P>(
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
  type ReqId = ();
}

struct CustomTransportParams((), ());

impl TransportParams for CustomTransportParams {
  type ExternalRequestParams = ();
  type ExternalResponseParams = ();

  fn ext_params(&self) -> (&Self::ExternalRequestParams, &Self::ExternalResponseParams) {
    (&self.0, &self.1)
  }

  fn ext_params_mut(&mut self) -> (&mut Self::ExternalRequestParams, &mut Self::ExternalResponseParams) {
    (&mut self.0, &mut self.1)
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