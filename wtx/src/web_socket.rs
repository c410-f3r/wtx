//! A computer communications protocol, providing full-duplex communication channels over a single
//! TCP connection.

mod close_code;
pub mod compression;
mod frame;
#[cfg(feature = "web-socket-handshake")]
mod handshake;
mod misc;
mod op_code;
mod unmask;
mod web_socket_buffer;
mod web_socket_error;
mod web_socket_parts;
mod web_socket_reader;
mod web_socket_writer;
use crate::{
  misc::{ConnectionState, LeaseMut, Rng, Stream},
  web_socket::compression::NegotiatedCompression,
  _MAX_PAYLOAD_LEN,
};
pub use close_code::CloseCode;
pub use compression::{Compression, CompressionLevel, DeflateConfig};
use core::marker::PhantomData;
pub use frame::{
  Frame, FrameControlArray, FrameControlArrayMut, FrameMut, FrameRef, FrameVec, FrameVecMut,
  FrameVecRef,
};
pub use misc::close_payload;
pub use op_code::OpCode;
pub use web_socket_buffer::WebSocketBuffer;
pub use web_socket_error::WebSocketError;
pub use web_socket_parts::{WebSocketCommonPart, WebSocketReaderPart, WebSocketWriterPart};

const MAX_CONTROL_PAYLOAD_LEN: usize = 125;
const MAX_HEADER_LEN_USIZE: usize = 14;

/// Always masks the payload before sending.
pub type WebSocketClient<NC, RNG, S, WSB> = WebSocket<NC, RNG, S, WSB, true>;
/// [`WebSocketClient`] with a mutable reference of [`WebSocketBuffer`].
pub type WebSocketClientMut<'wsb, NC, RNG, S> =
  WebSocketClient<NC, RNG, S, &'wsb mut WebSocketBuffer>;
/// [`WebSocketClient`] with an owned [`WebSocketBuffer`].
pub type WebSocketClientOwned<NC, RNG, S> = WebSocketClient<NC, RNG, S, WebSocketBuffer>;
/// Always unmasks the payload after receiving.
pub type WebSocketServer<NC, RNG, S, WSB> = WebSocket<NC, RNG, S, WSB, false>;
/// [`WebSocketServer`] with a mutable reference of [`WebSocketBuffer`].
pub type WebSocketServerMut<'wsb, NC, RNG, S> =
  WebSocketServer<NC, RNG, S, &'wsb mut WebSocketBuffer>;
/// [`WebSocketServer`] with an owned [`WebSocketBuffer`].
pub type WebSocketServerOwned<NC, RNG, S> = WebSocketServer<NC, RNG, S, WebSocketBuffer>;

/// Protocol implementation over an asynchronous stream.
///
/// <https://tools.ietf.org/html/rfc6455>
#[derive(Debug)]
pub struct WebSocket<NC, RNG, S, WSB, const IS_CLIENT: bool> {
  connection_state: ConnectionState,
  max_payload_len: usize,
  nc: NC,
  rng: RNG,
  stream: S,
  wsb: WSB,
}

impl<NC, RNG, S, WSB, const IS_CLIENT: bool> WebSocket<NC, RNG, S, WSB, IS_CLIENT> {
  /// Sets whether to automatically close the connection when a received frame payload length
  /// exceeds `max_payload_len`. Defaults to `64 * 1024 * 1024` bytes (64 MiB).
  #[inline]
  pub fn set_max_payload_len(&mut self, max_payload_len: usize) {
    self.max_payload_len = max_payload_len;
  }
}

impl<NC, RNG, S, WSB, const IS_CLIENT: bool> WebSocket<NC, RNG, S, WSB, IS_CLIENT>
where
  NC: NegotiatedCompression,
  RNG: Rng,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  /// Creates a new instance from a stream that supposedly has already completed the handshake.
  #[inline]
  pub fn new(nc: NC, rng: RNG, stream: S, mut wsb: WSB) -> crate::Result<Self> {
    wsb.lease_mut().network_buffer._clear_if_following_is_empty();
    wsb.lease_mut().network_buffer._reserve(MAX_HEADER_LEN_USIZE)?;
    Ok(Self {
      connection_state: ConnectionState::Open,
      max_payload_len: _MAX_PAYLOAD_LEN,
      nc,
      rng,
      stream,
      wsb,
    })
  }

  /// Different mutable parts that allow sending received frames using the same original instance.
  #[inline]
  pub fn parts(
    &mut self,
  ) -> (
    WebSocketCommonPart<'_, NC, RNG, S, IS_CLIENT>,
    WebSocketReaderPart<'_, NC, RNG, S, IS_CLIENT>,
    WebSocketWriterPart<'_, NC, RNG, S, IS_CLIENT>,
  ) {
    let WebSocket { connection_state, nc, rng, stream, wsb, max_payload_len } = self;
    let WebSocketBuffer {
      writer_buffer,
      network_buffer,
      reader_buffer_first,
      reader_buffer_second,
    } = wsb.lease_mut();
    (
      WebSocketCommonPart { connection_state, nc, rng, stream },
      WebSocketReaderPart {
        max_payload_len: *max_payload_len,
        network_buffer,
        phantom: PhantomData,
        reader_buffer_first,
        reader_buffer_second,
      },
      WebSocketWriterPart { phantom: PhantomData, writer_buffer },
    )
  }

  /// Reads a frame from the stream.
  ///
  /// If a frame is made up of other sub-frames or continuations, then everything is collected
  /// until all fragments are received.
  #[inline]
  pub async fn read_frame(&mut self) -> crate::Result<FrameMut<'_, IS_CLIENT>> {
    let WebSocket { connection_state, max_payload_len, nc, rng, stream, wsb } = self;
    let WebSocketBuffer {
      network_buffer,
      reader_buffer_first,
      reader_buffer_second,
      writer_buffer: _,
    } = wsb.lease_mut();
    web_socket_reader::read_frame_from_stream(
      connection_state,
      *max_payload_len,
      nc,
      network_buffer,
      reader_buffer_first,
      reader_buffer_second,
      rng,
      stream,
    )
    .await
  }

  /// Writes a frame to the stream.
  #[inline]
  pub async fn write_frame<P>(&mut self, frame: &mut Frame<P, IS_CLIENT>) -> crate::Result<()>
  where
    P: LeaseMut<[u8]>,
  {
    let WebSocket { connection_state, nc, rng, stream, wsb, .. } = self;
    let WebSocketBuffer { writer_buffer, .. } = wsb.lease_mut();
    web_socket_writer::write_frame(connection_state, frame, nc, rng, stream, writer_buffer).await
  }
}
