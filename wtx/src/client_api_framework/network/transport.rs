//! Implementations of the [Transport] trait.

mod mock;
mod recieving_transport;
mod sending_receiving_transport;
mod sending_transport;
mod transport_params;
mod unit;
#[cfg(feature = "http-client-pool")]
mod wtx_http;
#[cfg(feature = "web-socket")]
mod wtx_ws;

use crate::client_api_framework::network::TransportGroup;
pub use mock::{Mock, MockBytes, MockStr};
pub use recieving_transport::RecievingTransport;
pub use sending_receiving_transport::SendingReceivingTransport;
pub use sending_transport::SendingTransport;
pub use transport_params::TransportParams;

/// Any means of transferring data between two parties.
///
/// Please, see the [`Package`] implementation of the desired package to know
/// more about the expected types as well as any other additional documentation.
pub trait Transport<TP> {
  /// See [TransportGroup].
  const GROUP: TransportGroup;
  /// The inner implementation.
  type Inner: Transport<TP>;

  /// Instance counterpart of [`Self::GROUP`].
  #[inline]
  fn ty(&self) -> TransportGroup {
    Self::GROUP
  }
}

impl<T, TP> Transport<TP> for &mut T
where
  T: Transport<TP>,
{
  const GROUP: TransportGroup = T::GROUP;
  type Inner = T::Inner;
}

#[cfg(test)]
mod tests {
  use crate::{
    client_api_framework::pkg::Package,
    data_transformation::dnsn::{Deserialize, Serialize},
    misc::Vector,
  };

  #[derive(Debug, Eq, PartialEq)]
  pub(crate) struct _PingPong(pub(crate) _Ping, pub(crate) ());

  impl<DRSR, T, TP> Package<(), DRSR, T, TP> for _PingPong {
    type ExternalRequestContent = _Ping;
    type ExternalResponseContent<'de> = _Pong;
    type PackageParams = ();

    #[inline]
    fn ext_req_content(&self) -> &Self::ExternalRequestContent {
      &self.0
    }

    #[inline]
    fn ext_req_content_mut(&mut self) -> &mut Self::ExternalRequestContent {
      &mut self.0
    }

    #[inline]
    fn pkg_params(&self) -> &Self::PackageParams {
      &self.1
    }

    #[inline]
    fn pkg_params_mut(&mut self) -> &mut Self::PackageParams {
      &mut self.1
    }
  }

  #[derive(Debug, Eq, PartialEq)]
  pub(crate) struct _Ping;

  impl<DRSR> Serialize<DRSR> for _Ping {
    #[inline]
    fn to_bytes(&mut self, bytes: &mut Vector<u8>, _: &mut DRSR) -> crate::Result<()> {
      bytes.extend_from_copyable_slice(b"ping")?;
      Ok(())
    }
  }

  #[derive(Debug, Eq, PartialEq)]
  pub(crate) struct _Pong(pub(crate) &'static str);

  impl<'de, DRSR> Deserialize<'de, DRSR> for _Pong {
    #[inline]
    fn from_bytes(bytes: &[u8], _: &mut DRSR) -> crate::Result<Self> {
      assert_eq!(bytes, b"ping");
      Ok(Self("pong"))
    }

    #[inline]
    fn seq_from_bytes(_: &mut Vector<Self>, _: &'de [u8], _: &mut DRSR) -> crate::Result<()> {
      Ok(())
    }
  }
}
