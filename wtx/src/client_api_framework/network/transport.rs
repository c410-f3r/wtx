//! Implementations of the [Transport] trait.

mod mock;
mod recieving_transport;
mod sending_receiving_transport;
mod sending_transport;
mod transport_params;
mod unit;
#[cfg(feature = "http2")]
mod wtx_http;
#[cfg(feature = "web-socket")]
mod wtx_ws;

use crate::client_api_framework::network::TransportGroup;
pub use mock::{Mock, MockBytes, MockStr};
pub use recieving_transport::ReceivingTransport;
pub use sending_receiving_transport::SendingReceivingTransport;
pub use sending_transport::SendingTransport;
pub use transport_params::TransportParams;

/// Any means of transferring data between two parties.
///
/// Please, see the [`crate::client_api_framework::pkg::Package`] implementation of the desired package to know
/// more about the expected types as well as any other additional documentation.
pub trait Transport<TP> {
  /// See [TransportGroup].
  const GROUP: TransportGroup;
  /// The inner implementation.
  type Inner: Transport<TP>;
  /// If applicable, can be used by clients to poll specific sent requests.
  type ReqId;

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
  type ReqId = T::ReqId;
}

/// Used in [`crate::network::transport::Transport::send_recv_decode_contained`] and all implementations of
/// [`crate::Requests::decode_responses`].
///
/// Not used in [`crate::network::transport::Transport::send_recv_decode_batch`] because
/// [`crate::Requests::decode_responses`] takes precedence.
#[cfg(any(feature = "http2", feature = "web-socket"))]
pub(crate) fn log_res(
  _bytes: &[u8],
  _log_body: bool,
  _tg: TransportGroup,
  _uri: Option<&crate::misc::UriString>,
) {
  let _body = if _log_body { crate::misc::from_utf8_basic(_bytes).ok() } else { None };
  _debug!(trans_ty = display(_tg), r#"Response ({_uri:?}) "{_body:?}"#);
}

#[cfg(feature = "tokio")]
mod tokio {
  use crate::client_api_framework::network::{TransportGroup, transport::Transport};
  use tokio::sync::MappedMutexGuard;

  impl<T, TP> Transport<TP> for MappedMutexGuard<'_, T>
  where
    T: Transport<TP>,
  {
    const GROUP: TransportGroup = T::GROUP;
    type Inner = T::Inner;
    type ReqId = T::ReqId;
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    client_api_framework::pkg::Package,
    collection::Vector,
    de::{
      Decode, DecodeSeq, Encode,
      format::{De, DecodeWrapper, EncodeWrapper},
    },
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

  impl<DRSR> Encode<De<DRSR>> for _Ping {
    #[inline]
    fn encode(&self, _: &mut DRSR, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
      ew.vector.extend_from_copyable_slice(b"ping")?;
      Ok(())
    }
  }

  #[derive(Debug, Eq, PartialEq)]
  pub(crate) struct _Pong(pub(crate) &'static str);

  impl<'de, DRSR> Decode<'de, De<DRSR>> for _Pong {
    #[inline]
    fn decode(_: &mut DRSR, _: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
      Ok(Self("pong"))
    }
  }

  impl<'de, DRSR> DecodeSeq<'de, De<DRSR>> for _Pong {
    #[inline]
    fn decode_seq(
      _: &mut DRSR,
      _: &mut Vector<Self>,
      _: &mut DecodeWrapper<'de>,
    ) -> crate::Result<()> {
      Ok(())
    }
  }
}
