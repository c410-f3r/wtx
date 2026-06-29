//! <https://datatracker.ietf.org/doc/html/rfc7692>

mod deflate_config;
mod web_socket_decompression;
mod window_bits;
#[cfg(feature = "zlib-rs")]
mod zlib_rs;

use crate::{codec::Compression, collections::ArrayVectorU8, http::GenericHeader};
pub use deflate_config::DeflateConfig;
pub use web_socket_decompression::WebSocketDecompression;
pub use window_bits::WindowBits;
#[cfg(feature = "zlib-rs")]
pub use zlib_rs::{
  NegotiatedZlibRs, NegotiatedZlibRsCompression, NegotiatedZlibRsDecompression, ZlibRs,
};

/// Initial compression parameters defined before a handshake.
pub trait WsCompression<const IS_CLIENT: bool> {
  /// See [`NegotiatedWsCompression`].
  type NegotiatedCompression: NegotiatedWsCompression;

  /// Manages the defined parameters with the received parameters to decide which
  /// parameters will be settled.
  fn negotiate(
    self,
    headers: impl Iterator<Item = impl GenericHeader>,
  ) -> crate::Result<Self::NegotiatedCompression>;

  /// Requests headers bytes that will be sent to the server.
  fn req_headers(&self) -> ArrayVectorU8<u8, 160>;
}

impl<const IS_CLIENT: bool> WsCompression<IS_CLIENT> for () {
  type NegotiatedCompression = ();

  #[inline]
  fn negotiate(
    self,
    _: impl Iterator<Item = impl GenericHeader>,
  ) -> crate::Result<Self::NegotiatedCompression> {
    Ok(())
  }

  #[inline]
  fn req_headers(&self) -> ArrayVectorU8<u8, 160> {
    ArrayVectorU8::new()
  }
}

/// Final compression parameters defined after a handshake.
pub trait NegotiatedWsCompression: WebSocketCompression + WebSocketDecompression {
  /// See [`WebSocketCompression`].
  type Compression: WebSocketCompression;
  /// See [`WebSocketDecompression`].
  type Decompression: WebSocketDecompression;

  /// Splits itself.
  fn into_split(self) -> (Self::Compression, Self::Decompression);

  /// Response headers
  fn res_headers(&self) -> ArrayVectorU8<u8, 160>;

  /// Rsv1 bit
  fn rsv1(&self) -> u8;
}

impl NegotiatedWsCompression for () {
  type Compression = ();
  type Decompression = ();

  #[inline]
  fn into_split(self) -> (Self::Compression, Self::Decompression) {
    ((), ())
  }

  #[inline]
  fn res_headers(&self) -> ArrayVectorU8<u8, 160> {
    ArrayVectorU8::new()
  }

  #[inline]
  fn rsv1(&self) -> u8 {
    0
  }
}

impl<T> NegotiatedWsCompression for Option<T>
where
  T: NegotiatedWsCompression,
{
  type Compression = Option<T::Compression>;
  type Decompression = Option<T::Decompression>;

  #[inline]
  fn into_split(self) -> (Self::Compression, Self::Decompression) {
    match self {
      Some(el) => {
        let (compression, decompression) = el.into_split();
        (Some(compression), Some(decompression))
      }
      None => (None, None),
    }
  }

  #[inline]
  fn res_headers(&self) -> ArrayVectorU8<u8, 160> {
    match self {
      Some(el) => el.res_headers(),
      None => ArrayVectorU8::new(),
    }
  }

  #[inline]
  fn rsv1(&self) -> u8 {
    match self {
      Some(el) => el.rsv1(),
      None => ().rsv1(),
    }
  }
}

/// WebSocket Decompression
pub trait WebSocketCompression: Compression {
  /// No Content Takeover
  fn no_context_takeover(&self) -> bool;
}

impl WebSocketCompression for () {
  #[inline]
  fn no_context_takeover(&self) -> bool {
    false
  }
}

impl<T> WebSocketCompression for Option<T>
where
  T: WebSocketCompression,
{
  #[inline]
  fn no_context_takeover(&self) -> bool {
    if let Some(elem) = self { elem.no_context_takeover() } else { false }
  }
}
