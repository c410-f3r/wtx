//! A computer communications protocol, providing full-duplex communication channels over a single
//! TCP connection.

mod close_code;
pub mod compression;
mod frame;
#[cfg(feature = "web-socket-handshake")]
mod handshake;
mod is_in_continuation_frame;
mod misc;
mod op_code;
pub(crate) mod read_frame_info;
mod unmask;
#[cfg(feature = "web-socket-handshake")]
mod web_socket_acceptor;
mod web_socket_buffer;
#[cfg(feature = "web-socket-handshake")]
mod web_socket_connector;
mod web_socket_error;
mod web_socket_parts;
mod web_socket_read_mode;
pub(crate) mod web_socket_reader;
mod web_socket_replier;
pub(crate) mod web_socket_writer;

use crate::{
  _MAX_PAYLOAD_LEN,
  collection::Vector,
  misc::{ConnectionState, LeaseMut},
  rng::{Rng, SeedableRng},
  stream::Stream,
  sync::{Arc, AtomicBool},
};
pub use close_code::CloseCode;
pub use compression::{Compression, CompressionLevel, DeflateConfig};
use core::marker::PhantomData;
pub use frame::{
  Frame, FrameControlArray, FrameMut, FrameRef, FrameVector, FrameVectorMut, FrameVectorRef,
};
pub use misc::fill_with_close_code;
pub use op_code::OpCode;
#[cfg(feature = "web-socket-handshake")]
pub use web_socket_acceptor::WebSocketAcceptor;
pub use web_socket_buffer::WebSocketBuffer;
#[cfg(feature = "web-socket-handshake")]
pub use web_socket_connector::WebSocketConnector;
pub use web_socket_error::WebSocketError;
pub use web_socket_parts::{
  web_socket_mut::{WebSocketCommonMut, WebSocketReaderMut, WebSocketWriterMut},
  web_socket_owned::{WebSocketPartsOwned, WebSocketReaderOwned, WebSocketWriterOwned},
};
pub use web_socket_read_mode::WebSocketReadMode;
pub use web_socket_replier::WebSocketReplier;

const FIN_MASK: u8 = 0b1000_0000;
const MASK_MASK: u8 = 0b1000_0000;
const MAX_CONTROL_PAYLOAD_LEN: usize = 125;
const MAX_HEADER_LEN: usize = 14;
const OP_CODE_MASK: u8 = 0b0000_1111;
const PAYLOAD_MASK: u8 = 0b0111_1111;
const RSV1_MASK: u8 = 0b0100_0000;
const RSV2_MASK: u8 = 0b0010_0000;
const RSV3_MASK: u8 = 0b0001_0000;

/// [`WebSocket`] with a mutable reference of [`WebSocketBuffer`].
pub type WebSocketMut<'wsb, NC, R, S, const IS_CLIENT: bool> =
  WebSocket<NC, R, S, &'wsb mut WebSocketBuffer, IS_CLIENT>;
/// [`WebSocket`] with an owned [`WebSocketBuffer`].
pub type WebSocketOwned<NC, R, S, const IS_CLIENT: bool> =
  WebSocket<NC, R, S, WebSocketBuffer, IS_CLIENT>;

/// Full-duplex communication over an asynchronous stream.
///
/// <https://tools.ietf.org/html/rfc6455>
#[derive(Debug)]
pub struct WebSocket<NC, R, S, WSB, const IS_CLIENT: bool> {
  connection_state: ConnectionState,
  is_in_continuation_frame: Option<is_in_continuation_frame::IsInContinuationFrame>,
  max_payload_len: usize,
  nc: NC,
  nc_rsv1: u8,
  no_masking: bool,
  rng: R,
  stream: S,
  wsb: WSB,
}

impl<NC, R, S, WSB, const IS_CLIENT: bool> WebSocket<NC, R, S, WSB, IS_CLIENT> {
  /// Sets whether to automatically close the connection when a received frame payload length
  /// exceeds `max_payload_len`. Defaults to `64 * 1024 * 1024` bytes (64 MiB).
  #[inline]
  pub fn set_max_payload_len(&mut self, max_payload_len: usize) {
    self.max_payload_len = max_payload_len;
  }
}

impl<NC, R, S, WSB, const IS_CLIENT: bool> WebSocket<NC, R, S, WSB, IS_CLIENT>
where
  NC: compression::NegotiatedCompression,
  R: Rng,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  /// Creates a new instance from a stream that supposedly has already completed the handshake.
  #[inline]
  pub fn new(nc: NC, no_masking: bool, rng: R, stream: S, wsb: WSB) -> crate::Result<Self> {
    let nc_rsv1 = nc.rsv1();
    Ok(Self {
      connection_state: ConnectionState::Open,
      is_in_continuation_frame: None,
      max_payload_len: _MAX_PAYLOAD_LEN,
      nc,
      nc_rsv1,
      no_masking,
      rng,
      stream,
      wsb,
    })
  }

  /// Different mutable parts that allow sending received frames using common elements.
  #[inline]
  pub fn parts_mut(
    &mut self,
  ) -> (
    WebSocketCommonMut<'_, NC, R, S, IS_CLIENT>,
    WebSocketReaderMut<'_, NC, R, S, IS_CLIENT>,
    WebSocketWriterMut<'_, NC, R, S, IS_CLIENT>,
  ) {
    let WebSocket {
      connection_state,
      is_in_continuation_frame,
      nc,
      nc_rsv1,
      no_masking,
      rng,
      stream,
      wsb,
      max_payload_len,
    } = self;
    let WebSocketBuffer { network_buffer, reader_buffer, writer_buffer } = wsb.lease_mut();
    (
      WebSocketCommonMut { connection_state, nc, nc_rsv1: *nc_rsv1, rng, stream },
      WebSocketReaderMut {
        is_in_continuation_frame,
        phantom: PhantomData,
        wsrp: web_socket_parts::web_socket_generic::WebSocketReaderGeneric {
          max_payload_len: *max_payload_len,
          network_buffer,
          no_masking: *no_masking,
          reader_buffer,
        },
      },
      WebSocketWriterMut {
        phantom: PhantomData,
        wswp: web_socket_parts::web_socket_generic::WebSocketWriterGeneric {
          no_masking: *no_masking,
          writer_buffer,
        },
      },
    )
  }

  /// Reads a frame from the stream.
  ///
  /// If a frame is made up of other sub-frames or continuations, then everything is collected
  /// until all fragments are received.
  #[inline]
  pub async fn read_frame<'buffer, 'frame, 'this>(
    &'this mut self,
    buffer: &'buffer mut Vector<u8>,
    read_mode: WebSocketReadMode,
  ) -> crate::Result<FrameMut<'frame, IS_CLIENT>>
  where
    'buffer: 'frame,
    'this: 'frame,
  {
    let WebSocket {
      connection_state,
      is_in_continuation_frame,
      max_payload_len,
      nc,
      nc_rsv1,
      no_masking,
      rng,
      stream,
      wsb,
    } = self;
    let WebSocketBuffer { network_buffer, reader_buffer, writer_buffer: _ } = wsb.lease_mut();
    web_socket_reader::read_frame::<_, _, _, _, _, true, IS_CLIENT>(
      connection_state,
      is_in_continuation_frame,
      *max_payload_len,
      nc,
      *nc_rsv1,
      network_buffer,
      *no_masking,
      read_mode,
      reader_buffer,
      &WebSocketReplier::new(),
      rng,
      stream,
      buffer,
      |local_stream| local_stream,
      |local_stream| local_stream,
    )
    .await
  }

  /// Writes a frame to the stream.
  #[inline]
  pub async fn write_frame<P>(&mut self, frame: &mut Frame<P, IS_CLIENT>) -> crate::Result<()>
  where
    P: LeaseMut<[u8]>,
  {
    let WebSocket { connection_state, nc, nc_rsv1, no_masking, rng, stream, wsb, .. } = self;
    let WebSocketBuffer { writer_buffer, .. } = wsb.lease_mut();
    web_socket_writer::write_frame(
      connection_state,
      frame,
      *no_masking,
      nc,
      *nc_rsv1,
      rng,
      stream,
      writer_buffer,
    )
    .await?;
    Ok(())
  }
}

impl<NC, R, S, const IS_CLIENT: bool> WebSocket<NC, R, S, WebSocketBuffer, IS_CLIENT>
where
  NC: Clone + compression::NegotiatedCompression,
  R: Rng + SeedableRng,
{
  /// Splits the instance into owned parts that can be used in concurrent scenarios.
  #[inline]
  pub fn into_parts<SR, SW>(
    self,
    split: impl FnOnce(S) -> (SR, SW),
  ) -> crate::Result<WebSocketPartsOwned<NC, R, SR, SW, IS_CLIENT>> {
    let WebSocket {
      connection_state,
      is_in_continuation_frame,
      nc,
      nc_rsv1,
      no_masking,
      mut rng,
      stream,
      wsb,
      max_payload_len,
    } = self;
    let WebSocketBuffer { network_buffer, reader_buffer, writer_buffer } = wsb;
    let (stream_reader, stream_writer) = split(stream);
    let local_connection_state = Arc::new(AtomicBool::new(connection_state.into()));
    let replier = Arc::new(WebSocketReplier::new());
    Ok(WebSocketPartsOwned {
      reader: WebSocketReaderOwned {
        connection_state: local_connection_state.clone(),
        is_in_continuation_frame,
        phantom: PhantomData,
        nc: nc.clone(),
        nc_rsv1,
        reader_part: web_socket_parts::web_socket_generic::WebSocketReaderGeneric {
          max_payload_len,
          network_buffer,
          no_masking,
          reader_buffer,
        },
        replier: replier.clone(),
        rng: R::from_rng(&mut rng)?,
        stream_reader,
      },
      writer: WebSocketWriterOwned {
        connection_state: local_connection_state,
        nc,
        nc_rsv1,
        rng,
        stream_writer,
        writer_part: web_socket_parts::web_socket_generic::WebSocketWriterGeneric {
          no_masking,
          writer_buffer,
        },
      },
      replier,
    })
  }
}
