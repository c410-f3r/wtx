//! A computer communications protocol, providing full-duplex communication channels over a single
//! TCP connection.

#[macro_use]
mod macros;

mod close_code;
pub mod compression;
mod frame;
#[cfg(feature = "web-socket-handshake")]
mod handshake;
mod misc;
mod op_code;
mod read_frame_info;
mod unmask;
#[cfg(feature = "web-socket-handshake")]
mod web_socket_acceptor;
mod web_socket_buffer;
#[cfg(feature = "web-socket-handshake")]
mod web_socket_connector;
mod web_socket_error;
mod web_socket_parts;
pub(crate) mod web_socket_reader;
pub(crate) mod web_socket_writer;

use crate::{
  _MAX_PAYLOAD_LEN,
  misc::{ConnectionState, LeaseMut, Lock, Stream, Xorshift64},
  web_socket::{
    compression::NegotiatedCompression,
    web_socket_parts::web_socket_part::{
      WebSocketCommonPart, WebSocketReaderPart, WebSocketWriterPart,
    },
  },
};
pub use close_code::CloseCode;
pub use compression::{Compression, CompressionLevel, DeflateConfig};
use core::marker::PhantomData;
pub use frame::{
  Frame, FrameControlArray, FrameControlArrayMut, FrameMut, FrameRef, FrameVector, FrameVectorMut,
  FrameVectorRef,
};
pub use misc::fill_with_close_code;
pub use op_code::OpCode;
pub use read_frame_info::ReadFrameInfo;
#[cfg(feature = "web-socket-handshake")]
pub use web_socket_acceptor::WebSocketAcceptor;
pub use web_socket_buffer::WebSocketBuffer;
#[cfg(feature = "web-socket-handshake")]
pub use web_socket_connector::WebSocketConnector;
pub use web_socket_error::WebSocketError;
pub use web_socket_parts::{
  web_socket_part_mut::{WebSocketCommonPartMut, WebSocketReaderPartMut, WebSocketWriterPartMut},
  web_socket_part_owned::{
    WebSocketCommonPartOwned, WebSocketPartsOwned, WebSocketReaderPartOwned,
    WebSocketWriterPartOwned,
  },
};

const FIN_MASK: u8 = 0b1000_0000;
const MASK_MASK: u8 = 0b1000_0000;
const MAX_CONTROL_PAYLOAD_LEN: usize = 125;
const MAX_HEADER_LEN_USIZE: usize = 14;
const OP_CODE_MASK: u8 = 0b0000_1111;
const PAYLOAD_MASK: u8 = 0b0111_1111;
const RSV1_MASK: u8 = 0b0100_0000;
const RSV2_MASK: u8 = 0b0010_0000;
const RSV3_MASK: u8 = 0b0001_0000;

/// [`WebSocketClient`] with a mutable reference of [`WebSocketBuffer`].
pub type WebSocketMut<'wsb, NC, S, const IS_CLIENT: bool> =
  WebSocket<NC, S, &'wsb mut WebSocketBuffer, IS_CLIENT>;
/// [`WebSocketClient`] with an owned [`WebSocketBuffer`].
pub type WebSocketOwned<NC, S, const IS_CLIENT: bool> =
  WebSocket<NC, S, WebSocketBuffer, IS_CLIENT>;

/// Full-duplex communication over an asynchronous stream.
///
/// <https://tools.ietf.org/html/rfc6455>
#[derive(Debug)]
pub struct WebSocket<NC, S, WSB, const IS_CLIENT: bool> {
  connection_state: ConnectionState,
  max_payload_len: usize,
  nc: NC,
  no_masking: bool,
  rng: Xorshift64,
  stream: S,
  wsb: WSB,
}

impl<NC, S, WSB, const IS_CLIENT: bool> WebSocket<NC, S, WSB, IS_CLIENT> {
  /// Sets whether to automatically close the connection when a received frame payload length
  /// exceeds `max_payload_len`. Defaults to `64 * 1024 * 1024` bytes (64 MiB).
  #[inline]
  pub fn set_max_payload_len(&mut self, max_payload_len: usize) {
    self.max_payload_len = max_payload_len;
  }
}

impl<NC, S, WSB, const IS_CLIENT: bool> WebSocket<NC, S, WSB, IS_CLIENT>
where
  NC: NegotiatedCompression,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  /// Creates a new instance from a stream that supposedly has already completed the handshake.
  #[inline]
  pub const fn new(
    nc: NC,
    no_masking: bool,
    rng: Xorshift64,
    stream: S,
    wsb: WSB,
  ) -> crate::Result<Self> {
    Ok(Self {
      connection_state: ConnectionState::Open,
      max_payload_len: _MAX_PAYLOAD_LEN,
      nc,
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
    WebSocketCommonPartMut<'_, NC, S, IS_CLIENT>,
    WebSocketReaderPartMut<'_, NC, S, IS_CLIENT>,
    WebSocketWriterPartMut<'_, NC, S, IS_CLIENT>,
  ) {
    let WebSocket { connection_state, nc, no_masking, rng, stream, wsb, max_payload_len } = self;
    let WebSocketBuffer {
      writer_buffer,
      network_buffer,
      reader_buffer_first,
      reader_buffer_second,
    } = wsb.lease_mut();
    let nc_rsv1 = nc.rsv1();
    (
      WebSocketCommonPartMut { wsc: WebSocketCommonPart { connection_state, nc, rng, stream } },
      WebSocketReaderPartMut {
        phantom: PhantomData,
        wsrp: WebSocketReaderPart {
          max_payload_len: *max_payload_len,
          nc_rsv1,
          network_buffer,
          no_masking: *no_masking,
          reader_buffer_first,
          reader_buffer_second,
        },
      },
      WebSocketWriterPartMut {
        phantom: PhantomData,
        wswp: WebSocketWriterPart { no_masking: *no_masking, writer_buffer },
      },
    )
  }

  /// Reads a frame from the stream.
  ///
  /// If a frame is made up of other sub-frames or continuations, then everything is collected
  /// until all fragments are received.
  #[inline]
  pub async fn read_frame(&mut self) -> crate::Result<FrameMut<'_, IS_CLIENT>> {
    let WebSocket { connection_state, max_payload_len, nc, no_masking, rng, stream, wsb } = self;
    let WebSocketBuffer {
      network_buffer,
      reader_buffer_first,
      reader_buffer_second,
      writer_buffer: _,
    } = wsb.lease_mut();
    let nc_rsv1 = nc.rsv1();
    let frame = read_frame!(
      *max_payload_len,
      (NC::IS_NOOP, nc_rsv1),
      network_buffer,
      *no_masking,
      &mut *reader_buffer_first,
      reader_buffer_second,
      stream,
      (
        stream,
        WebSocketCommonPart::<_, _, _, _, IS_CLIENT> {
          connection_state: &mut *connection_state,
          nc: &mut *nc,
          rng: &mut *rng,
          stream: &mut *stream
        }
      )
    );
    Ok(frame)
  }

  /// Writes a frame to the stream.
  #[inline]
  pub async fn write_frame<P>(&mut self, frame: &mut Frame<P, IS_CLIENT>) -> crate::Result<()>
  where
    P: LeaseMut<[u8]>,
  {
    let WebSocket { connection_state, nc, no_masking, rng, stream, wsb, .. } = self;
    let WebSocketBuffer { writer_buffer, .. } = wsb.lease_mut();
    web_socket_writer::write_frame(
      connection_state,
      frame,
      *no_masking,
      nc,
      rng,
      stream,
      writer_buffer,
    )
    .await?;
    Ok(())
  }
}

impl<NC, S, const IS_CLIENT: bool> WebSocket<NC, S, WebSocketBuffer, IS_CLIENT>
where
  NC: NegotiatedCompression,
{
  /// Splits the instance into owned parts that can be used in concurrent scenarios.
  #[inline]
  pub fn into_parts<C, SR, SW>(
    self,
    split: impl FnOnce(S) -> (SR, SW),
  ) -> WebSocketPartsOwned<C, NC, SR, SW, IS_CLIENT>
  where
    C: Clone + Lock<Resource = WebSocketCommonPartOwned<NC, SW, IS_CLIENT>>,
  {
    let WebSocket { connection_state, nc, no_masking, rng, stream, wsb, max_payload_len } = self;
    let WebSocketBuffer {
      writer_buffer,
      network_buffer,
      reader_buffer_first,
      reader_buffer_second,
    } = wsb;
    let (stream_reader, stream_writer) = split(stream);
    let nc_rsv1 = nc.rsv1();
    let common = C::new(WebSocketCommonPartOwned {
      wsc: WebSocketCommonPart { connection_state, nc, rng, stream: stream_writer },
    });
    WebSocketPartsOwned {
      reader: WebSocketReaderPartOwned {
        common: common.clone(),
        phantom: PhantomData,
        stream_reader,
        wsrp: WebSocketReaderPart {
          max_payload_len,
          nc_rsv1,
          network_buffer,
          no_masking,
          reader_buffer_first,
          reader_buffer_second,
        },
      },
      writer: WebSocketWriterPartOwned {
        common,
        phantom: PhantomData,
        wswp: WebSocketWriterPart { no_masking, writer_buffer },
      },
    }
  }
}
