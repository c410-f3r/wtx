//! HTTP/2

// Many elements where influenced by the code of the h2 repository (https://github.com/hyperium/h2)
// so thanks to the authors.

mod client_stream;
mod data_frame;
mod error_code;
mod frame;
mod frame_init;
mod go_away_frame;
mod headers_frame;
mod hpack_decoder;
mod hpack_encoder;
mod hpack_header;
mod hpack_static_headers;
mod http2_buffer;
mod http2_data;
mod huffman;
mod huffman_tables;
mod misc;
mod params;
mod ping_frame;
mod req_res_buffer;
mod reset_frame;
mod server_stream;
mod settings_frame;
mod stream_data;
mod stream_id;
mod stream_state;
#[cfg(test)]
mod tests;
mod uri_buffer;
mod window_update_frame;

use crate::{
  http::Headers,
  misc::{Lock, RefCounter, SingleTypeStorage},
};
use alloc::sync::Arc;
pub use client_stream::ClientStream;
use core::{
  marker::PhantomData,
  sync::atomic::{AtomicBool, Ordering},
};
pub(crate) use data_frame::DataFrame;
pub(crate) use error_code::ErrorCode;
pub(crate) use frame::Frame;
pub(crate) use frame_init::{FrameHeaderTy, FrameInit};
pub(crate) use go_away_frame::GoAwayFrame;
pub(crate) use headers_frame::HeadersFrame;
pub(crate) use hpack_decoder::HpackDecoder;
pub(crate) use hpack_encoder::HpackEncoder;
pub(crate) use hpack_header::HpackHeaderBasic;
pub(crate) use hpack_static_headers::{HpackStaticRequestHeaders, HpackStaticResponseHeaders};
pub use http2_buffer::Http2Buffer;
pub(crate) use http2_data::Http2Data;
pub(crate) use huffman::{huffman_decode, huffman_encode};
pub use params::{AcceptParams, ConnectParams};
pub(crate) use ping_frame::PingFrame;
pub use req_res_buffer::ReqResBuffer;
pub(crate) use reset_frame::ResetFrame;
pub use server_stream::ServerStream;
pub(crate) use settings_frame::SettingsFrame;
pub(crate) use stream_data::StreamData;
pub(crate) use stream_id::StreamId;
pub(crate) use stream_state::StreamState;
pub(crate) use uri_buffer::UriBuffer;
pub(crate) use window_update_frame::WindowUpdateFrame;

pub const DEFAULT_MAX_HEADER_LIST_SIZE: u32 = 16777216;
/// The default value of SETTINGS_INITIAL_WINDOW_SIZE
pub const DEFAULT_INITIAL_WINDOW_SIZE: u32 = 65535;
/// The default value of SETTINGS_HEADER_TABLE_SIZE
pub const DEFAULT_MAX_HEADER_TABLE_SIZE: u32 = 4_096;
/// Maximum frame size can not be lower than this value.
pub const MAX_FRAME_SIZE_LOWER_BOUND: u32 = 16384;
/// Maximum frame size can not be higher than this value.
pub const MAX_FRAME_SIZE_UPPER_BOUND: u32 = 16777215;
/// INITIAL_WINDOW_SIZE upper bound
pub const MAX_INITIAL_WINDOW_SIZE: u32 = 2147483647;

const ACK_MASK: u8 = 0b0000_0001;
const PAD_MASK: u8 = 0b0000_1000;
const PREFACE: &[u8; 24] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

/// Http2 instance using the mutex from tokio.
#[cfg(feature = "tokio")]
pub type Http2Tokio<S, const IS_CLIENT: bool> =
  Http2<S, Arc<tokio::sync::Mutex<Http2Data<S, IS_CLIENT>>>, IS_CLIENT>;

#[derive(Debug)]
pub struct Http2<S, SDC, const IS_CLIENT: bool> {
  is_closed: AtomicBool,
  phantom: PhantomData<S>,
  sdrc: SDC,
  stream_id: StreamId,
}

impl<S, SDC, SDL, const IS_CLIENT: bool> Http2<S, SDC, IS_CLIENT>
where
  S: crate::misc::Stream,
  SDC: RefCounter<SDL> + SingleTypeStorage<Item = SDL>,
  SDL: Lock<Http2Data<S, false>>,
{
  pub fn is_closed(&self) -> bool {
    self.is_closed.load(Ordering::Relaxed)
  }
}

impl<S, SDC, SDL> Http2<S, SDC, false>
where
  S: crate::misc::Stream,
  SDC: RefCounter<SDL> + SingleTypeStorage<Item = SDL>,
  SDL: Lock<Http2Data<S, false>>,
{
  #[inline]
  pub async fn accept(
    ap: AcceptParams,
    mut hb: Http2Buffer<false>,
    mut stream: S,
  ) -> crate::Result<Self> {
    hb.clear();
    stream.read(&mut hb.rb._following_trail_mut()).await?;
    if hb.rb._buffer() != PREFACE {
      return Err(crate::Error::NoPreface);
    }
    hb.rb._clear();
    stream.write(PREFACE).await?;
    SettingsFrame::ack().write(&mut stream).await?;
    Ok(Self {
      is_closed: AtomicBool::new(false),
      phantom: PhantomData,
      sdrc: SDC::new(SDL::new(Http2Data::new(hb, stream))),
      stream_id: StreamId::from(0),
    })
  }

  #[inline]
  pub async fn recv_stream(
    &mut self,
    headers: &mut Headers,
  ) -> crate::Result<Option<ServerStream>> {
    Ok(Some(ServerStream::new()))
  }
}

impl<S, SDC, SDL> Http2<S, SDC, true>
where
  S: crate::misc::Stream,
  SDC: RefCounter<SDL> + SingleTypeStorage<Item = SDL>,
  SDL: Lock<Http2Data<S, true>>,
{
  #[inline]
  pub async fn connect(
    cp: ConnectParams,
    mut hb: Http2Buffer<true>,
    mut stream: S,
  ) -> crate::Result<Self> {
    hb.clear();
    stream.write(PREFACE).await?;
    let mut buffer = [0; 4];
    let _ = stream.read(&mut buffer).await?;
    if buffer != [0, 0, 0, 4] {
      return Err(crate::Error::NoAckSettings);
    }
    let mut hd = Http2Data::new(hb, stream);
    if let Some(elem) = cp.settings.max_frame_size() {
      hd.set_max_frame_size(elem);
    }
    if let Some(elem) = cp.settings.max_header_list_size() {
      hd.set_max_header_list_size(elem);
    }
    cp.settings.write(hd.stream_mut()).await?;
    Ok(Self {
      is_closed: AtomicBool::new(false),
      phantom: PhantomData,
      sdrc: SDC::new(SDL::new(hd)),
      stream_id: StreamId::from(1),
    })
  }

  pub async fn stream(&mut self) -> crate::Result<ClientStream<S, SDC, true>> {
    let rslt = ClientStream::new(self.sdrc.clone(), self.stream_id);
    self.stream_id = self.stream_id.wrapping_add(StreamId::from(2));
    Ok(rslt)
  }
}
