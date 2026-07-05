//! A computer communications protocol, providing full-duplex communication channels over a single
//! TCP connection.

mod close_code;
mod frame;
#[cfg(feature = "web-socket-handshake")]
mod handshake;
mod is_in_continuation_frame;
mod misc;
mod op_code;
pub(crate) mod read_frame;
pub(crate) mod read_frame_info;
mod unmask;
#[cfg(feature = "web-socket-handshake")]
mod web_socket_acceptor;
mod web_socket_bridge;
mod web_socket_buffer;
pub mod web_socket_compression;
#[cfg(feature = "web-socket-handshake")]
mod web_socket_connector;
mod web_socket_error;
pub(crate) mod web_socket_mut;
pub(crate) mod web_socket_owned;
mod web_socket_payload_origin;
pub(crate) mod write_frame;

use crate::{
  _MAX_PAYLOAD_LEN,
  collections::Vector,
  misc::{ConnectionState, LeaseMut},
  rng::{SeedableRng as _, Xorshift64},
  stream::Stream,
  tls::{TlsMode, TlsStream, TlsStreamBridge},
  web_socket::web_socket_compression::NegotiatedWsCompression,
};
pub use close_code::CloseCode;
use core::marker::PhantomData;
pub use frame::{
  Frame, FrameControlArray, FrameMut, FrameRef, FrameVector, FrameVectorMut, FrameVectorRef,
};
pub use op_code::OpCode;
#[cfg(feature = "web-socket-handshake")]
pub use web_socket_acceptor::WebSocketAcceptor;
pub use web_socket_bridge::{WebSocketBridge, WebSocketBridgeData};
pub use web_socket_buffer::WebSocketBuffer;
pub use web_socket_compression::{DeflateConfig, WsCompression};
#[cfg(feature = "web-socket-handshake")]
pub use web_socket_connector::WebSocketConnector;
pub use web_socket_error::WebSocketError;
pub use web_socket_mut::{WebSocketCommonMut, WebSocketReaderMut, WebSocketWriterMut};
pub use web_socket_owned::{WebSocketReaderOwned, WebSocketWriterOwned};
pub use web_socket_payload_origin::WebSocketPayloadOrigin;

const FIN_MASK: u8 = 0b1000_0000;
const MASK_MASK: u8 = 0b1000_0000;
const MAX_CONTROL_PAYLOAD_LEN: usize = 125;
const MAX_HEADER_LEN: usize = 14;
const OP_CODE_MASK: u8 = 0b0000_1111;
const PAYLOAD_MASK: u8 = 0b0111_1111;
const RSV1_MASK: u8 = 0b0100_0000;
const RSV2_MASK: u8 = 0b0010_0000;
const RSV3_MASK: u8 = 0b0001_0000;

type IntoSplitTy<NC, S, TM, const IS_CLIENT: bool> = (
  WebSocketBridge<IS_CLIENT>,
  WebSocketReaderOwned<
    <NC as NegotiatedWsCompression>::Decompression,
    <S as Stream>::ReadHalfOwned,
    TM,
    IS_CLIENT,
  >,
  WebSocketWriterOwned<
    <NC as NegotiatedWsCompression>::Compression,
    <S as Stream>::WriteHalfOwned,
    TM,
    IS_CLIENT,
  >,
);

/// Full-duplex communication over an asynchronous stream.
///
/// <https://tools.ietf.org/html/rfc6455>
#[derive(Debug)]
pub struct WebSocket<NC, S, TM, const IS_CLIENT: bool> {
  is_in_continuation_frame: Option<is_in_continuation_frame::IsInContinuationFrame>,
  max_payload_len: usize,
  nc: NC,
  nc_rsv1: u8,
  no_masking: bool,
  rng: Xorshift64,
  stream: TlsStream<S, TM, IS_CLIENT>,
  wsb: WebSocketBuffer,
}

impl<NC, S, TM, const IS_CLIENT: bool> WebSocket<NC, S, TM, IS_CLIENT> {
  /// Sets whether to automatically close the connection when a received frame payload length
  /// exceeds `max_payload_len`. Defaults to `64 * 1024 * 1024` bytes (64 MiB).
  #[inline]
  pub const fn max_payload_len_mut(&mut self) -> &mut usize {
    &mut self.max_payload_len
  }
}

impl<NC, S, TM, const IS_CLIENT: bool> WebSocket<NC, S, TM, IS_CLIENT>
where
  NC: NegotiatedWsCompression,
  S: Stream,
  TM: TlsMode,
{
  /// Creates a new instance from a stream that supposedly has already completed the handshake.
  #[inline]
  pub fn new(
    nc: NC,
    no_masking: bool,
    rng: Xorshift64,
    stream: TlsStream<S, TM, IS_CLIENT>,
    wsb: WebSocketBuffer,
  ) -> Self {
    let nc_rsv1 = nc.rsv1();
    Self {
      is_in_continuation_frame: None,
      max_payload_len: _MAX_PAYLOAD_LEN,
      nc,
      nc_rsv1,
      no_masking,
      rng,
      stream,
      wsb,
    }
  }

  /// Reads a frame from the stream.
  ///
  /// If a frame is made up of other sub-frames or continuations, then everything is collected
  /// until all fragments are received.
  #[inline]
  pub async fn read_frame<'buffer, 'frame, 'this>(
    &'this mut self,
    buffer: &'buffer mut Vector<u8>,
    payload_origin: WebSocketPayloadOrigin,
  ) -> crate::Result<FrameMut<'frame>>
  where
    'buffer: 'frame,
    'this: 'frame,
  {
    let WebSocket {
      is_in_continuation_frame,
      max_payload_len,
      nc,
      nc_rsv1,
      no_masking,
      rng,
      stream,
      wsb,
    } = self;
    let WebSocketBuffer { network_buffer, reader_buffer, .. } = wsb;
    read_frame::read_frame::<_, _, _, _, _, true, IS_CLIENT>(
      is_in_continuation_frame,
      *max_payload_len,
      nc,
      *nc_rsv1,
      network_buffer,
      *no_masking,
      payload_origin,
      reader_buffer,
      rng,
      stream,
      &WebSocketBridge::new(TlsStreamBridge::new()),
      buffer,
      |el| el.connection_state = ConnectionState::Closed,
      |local_stream| local_stream,
      |local_stream| local_stream,
    )
    .await
  }

  /// Different mutable parts that allow sending received frames using common elements.
  #[inline]
  pub fn split_mut(
    &mut self,
  ) -> (
    WebSocketCommonMut<'_, NC, S, TM, IS_CLIENT>,
    WebSocketReaderMut<'_, NC, S, TM, IS_CLIENT>,
    WebSocketWriterMut<'_, NC, S, TM, IS_CLIENT>,
  ) {
    let WebSocket {
      is_in_continuation_frame,
      nc,
      nc_rsv1,
      no_masking,
      rng,
      stream,
      wsb,
      max_payload_len,
    } = self;
    let WebSocketBuffer { network_buffer, reader_buffer, writer_buffer } = wsb;
    (
      WebSocketCommonMut { nc, nc_rsv1: *nc_rsv1, rng, stream },
      WebSocketReaderMut {
        is_in_continuation_frame,
        max_payload_len: *max_payload_len,
        network_buffer,
        no_masking: *no_masking,
        phantom: PhantomData,
        reader_buffer,
      },
      WebSocketWriterMut { no_masking: *no_masking, phantom: PhantomData, writer_buffer },
    )
  }

  /// Writes a frame to the stream.
  #[inline]
  pub async fn write_frame<P>(&mut self, frame: &mut Frame<P>) -> crate::Result<()>
  where
    P: LeaseMut<[u8]>,
  {
    write_frame::write_frame::<_, _, _, _, IS_CLIENT>(
      frame,
      self.no_masking,
      &mut self.nc,
      self.nc_rsv1,
      &mut self.rng,
      &mut self.stream,
      &mut self.wsb.writer_buffer,
      |el| el.connection_state = ConnectionState::WriteClosed,
    )
    .await
  }
}

impl<NC, S, TM, const IS_CLIENT: bool> WebSocket<NC, S, TM, IS_CLIENT>
where
  NC: NegotiatedWsCompression,
  S: Stream,
  TM: TlsMode,
{
  /// Splits this instance into owned parts that can be used in concurrent scenarios.
  #[inline]
  pub fn into_split(self) -> crate::Result<IntoSplitTy<NC, S, TM, IS_CLIENT>> {
    let WebSocket {
      is_in_continuation_frame,
      nc,
      nc_rsv1,
      no_masking,
      mut rng,
      stream,
      wsb,
      max_payload_len,
    } = self;
    let (compression, decompression) = nc.into_split();
    let WebSocketBuffer { network_buffer, reader_buffer, writer_buffer } = wsb;
    let (stream_bridge_tls, stream_reader, stream_writer) = stream.into_split()?;
    let stream_bridge_ws = WebSocketBridge::new(stream_bridge_tls);
    Ok((
      stream_bridge_ws.clone(),
      WebSocketReaderOwned {
        is_in_continuation_frame,
        max_payload_len,
        nc: decompression,
        nc_rsv1,
        network_buffer,
        no_masking,
        phantom: PhantomData,
        reader_buffer,
        rng: Xorshift64::from_rng(&mut rng)?,
        stream_bridge: stream_bridge_ws,
        stream_reader,
      },
      WebSocketWriterOwned {
        nc: compression,
        nc_rsv1,
        no_masking,
        rng,
        stream_writer,
        writer_buffer,
      },
    ))
  }
}
