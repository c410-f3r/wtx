//! HTTP/2
//!
//! 1. Does not support padded headers when writing.
//! 2. Does not support push promises (Deprecated by the RFC).
//! 3. Does not support prioritization (Deprecated by he RFC).

// Many elements where influenced by the code of the h2 repository (https://github.com/hyperium/h2)
// so thanks to the authors.

#[macro_use]
mod macros;

mod buffers;
mod client_stream;
mod continuation_frame;
mod data_frame;
mod frame_init;
mod go_away_frame;
mod headers_frame;
mod hpack_decoder;
mod hpack_encoder;
mod hpack_header;
mod hpack_static_headers;
mod http2_data;
mod http2_error;
mod http2_error_code;
mod http2_params;
mod http2_params_send;
mod huffman;
mod huffman_tables;
mod misc;
mod ping_frame;
mod process_receipt_frame_ty;
mod reset_stream_frame;
mod send_msg;
mod server_stream;
mod settings_frame;
mod stream_receiver;
mod stream_state;
#[cfg(test)]
mod tests;
mod u31;
mod uri_buffer;
mod window;
mod window_update_frame;

use crate::{
  http2::misc::apply_initial_params,
  misc::{ConnectionState, LeaseMut, Lock, RefCounter, Stream, Usize},
};
pub use buffers::{Http2Buffer, ReqResBuffer, StreamBuffer};
pub use client_stream::ClientStream;
pub(crate) use continuation_frame::ContinuationFrame;
pub(crate) use data_frame::DataFrame;
pub(crate) use frame_init::{FrameInit, FrameInitTy};
pub(crate) use go_away_frame::GoAwayFrame;
use hashbrown::HashMap;
pub(crate) use headers_frame::HeadersFrame;
pub(crate) use hpack_decoder::HpackDecoder;
pub(crate) use hpack_encoder::HpackEncoder;
pub(crate) use hpack_header::HpackHeaderBasic;
pub(crate) use hpack_static_headers::{HpackStaticRequestHeaders, HpackStaticResponseHeaders};
pub(crate) use http2_data::Http2Data;
pub use http2_error::Http2Error;
pub use http2_error_code::Http2ErrorCode;
pub use http2_params::Http2Params;
pub(crate) use huffman::{huffman_decode, huffman_encode};
pub(crate) use ping_frame::PingFrame;
pub(crate) use process_receipt_frame_ty::ProcessReceiptFrameTy;
pub(crate) use reset_stream_frame::ResetStreamFrame;
pub use server_stream::ServerStream;
pub(crate) use settings_frame::SettingsFrame;
pub(crate) use stream_receiver::{StreamControlRecvParams, StreamOverallRecvParams};
pub(crate) use stream_state::StreamState;
use tokio::sync::MutexGuard;
pub(crate) use u31::U31;
pub(crate) use uri_buffer::UriBuffer;
pub(crate) use window::Windows;
pub(crate) use window_update_frame::WindowUpdateFrame;

pub(crate) const MAX_BODY_LEN: u32 = max_body_len!();
pub(crate) const MAX_HPACK_LEN: u32 = max_hpack_len!();
pub(crate) const MAX_CONCURRENT_STREAMS_NUM: u32 = max_concurrent_streams_num!();
pub(crate) const MAX_HEADERS_LEN: u32 = max_headers_len!();
pub(crate) const MAX_FRAME_LEN: u32 = max_frame_len!();
pub(crate) const MAX_FRAME_LEN_LOWER_BOUND: u32 = max_frame_len_lower_bound!();
pub(crate) const MAX_FRAME_LEN_UPPER_BOUND: u32 = max_frame_len_upper_bound!();
pub(crate) const MAX_RECV_STREAMS_NUM: u32 = max_recv_streams_num!();
pub(crate) const READ_BUFFER_LEN: u32 = read_buffer_len!();

const ACK_MASK: u8 = 0b0000_0001;
const EOH_MASK: u8 = 0b0000_0100;
const EOS_MASK: u8 = 0b0000_0001;
const PAD_MASK: u8 = 0b0000_1000;
const PREFACE: &[u8; 24] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

/// Http2 instance using the mutex from tokio.
#[cfg(feature = "tokio")]
pub type Http2Tokio<HB, S, SB, const IS_CLIENT: bool> =
  Http2<alloc::sync::Arc<tokio::sync::Mutex<Http2Data<HB, S, SB, IS_CLIENT>>>, IS_CLIENT>;

pub(crate) type Sorp<SB> = HashMap<U31, StreamOverallRecvParams<SB>>;
pub(crate) type Scrp = HashMap<U31, StreamControlRecvParams>;

/// Negotiates initial "handshakes" or connections and also manages the creation of streams.
#[derive(Debug)]
pub struct Http2<HD, const IS_CLIENT: bool> {
  hd: HD,
}

impl<HB, HD, S, SB, const IS_CLIENT: bool> Http2<HD, IS_CLIENT>
where
  HB: LeaseMut<Http2Buffer<SB>>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, S, SB, IS_CLIENT>>,
  S: Stream,
  SB: LeaseMut<StreamBuffer>,
{
  /// See [ConnectionState].
  #[inline]
  pub async fn connection_state(&self) -> ConnectionState {
    ConnectionState::from(self.hd.lock().await.is_conn_open())
  }

  /// Sends a GOAWAY frame to the peer, which cancels the connection and consequently all ongoing
  /// streams.
  pub async fn send_go_away(self, error_code: Http2ErrorCode) {
    let mut guard = self.hd.lock().await;
    let hdpm = guard.parts_mut();
    misc::send_go_away(error_code, hdpm.is_conn_open, *hdpm.last_stream_id, hdpm.stream).await;
  }
}

impl<HB, HD, S, SB> Http2<HD, false>
where
  HB: LeaseMut<Http2Buffer<SB>>,
  HD: RefCounter,
  for<'guard> HD::Item: Lock<
      Guard<'guard> = MutexGuard<'guard, Http2Data<HB, S, SB, false>>,
      Resource = Http2Data<HB, S, SB, false>,
    > + 'guard,
  S: Stream,
  SB: LeaseMut<StreamBuffer>,
{
  /// Accepts an initial connection sending the local parameters to the remote peer.
  #[inline]
  pub async fn accept(mut hb: HB, hp: Http2Params, mut stream: S) -> crate::Result<Self> {
    hb.lease_mut().clear();
    let mut buffer = [0; 24];
    let _ = stream.read(&mut buffer).await?;
    if &buffer != PREFACE {
      return Err(crate::Error::http2_go_away_generic(Http2Error::NoPreface));
    }
    stream.write_all(hp.to_settings_frame().bytes(&mut [0; 45])).await?;
    apply_initial_params(hb.lease_mut(), &hp)?;
    Ok(Self { hd: HD::new(HD::Item::new(Http2Data::new(hb, hp, stream))) })
  }

  /// Awaits for an initial header to create a stream.
  #[inline]
  pub async fn stream(&mut self, mut sb: SB) -> crate::Result<ServerStream<HD>> {
    sb.lease_mut().clear();
    {
      let mut guard = self.hd.lock().await;
      let hdpm = guard.parts_mut();
      if *hdpm.recv_streams_num > hdpm.hp.max_recv_streams_num() {
        return Err(crate::Error::http2_go_away_generic(
          Http2Error::ExceedAmountOfActiveConcurrentStreams,
        ));
      }
      *hdpm.recv_streams_num = hdpm.recv_streams_num.wrapping_add(1);
      sb.lease_mut().rrb.headers.set_max_bytes(*Usize::from(hdpm.hp.max_hpack_len().0));
      hdpm.hb.initial_server_buffers.push(sb);
    }
    process_receipt_loop!(self.hd, |guard| {
      if let Some((method, stream_id)) = guard.parts_mut().hb.initial_server_streams.pop_back() {
        drop(guard);
        return Ok(ServerStream::new(
          self.hd.clone(),
          method,
          _trace_span!("Creating server stream", stream_id = stream_id.u32()),
          stream_id,
        ));
      }
    });
  }
}

impl<HB, HD, S, SB> Http2<HD, true>
where
  HB: LeaseMut<Http2Buffer<SB>>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, S, SB, true>>,
  S: Stream,
  SB: LeaseMut<StreamBuffer>,
{
  /// Tries to connect to a server sending the local parameters.
  #[inline]
  pub async fn connect(mut hb: HB, hp: Http2Params, mut stream: S) -> crate::Result<Self> {
    hb.lease_mut().clear();
    stream.write_all_vectored([PREFACE, hp.to_settings_frame().bytes(&mut [0; 45])]).await?;
    apply_initial_params(hb.lease_mut(), &hp)?;
    Ok(Self { hd: HD::new(HD::Item::new(Http2Data::new(hb, hp, stream))) })
  }

  /// Opens a local stream.
  pub async fn stream(&mut self) -> crate::Result<ClientStream<HD>> {
    let mut guard = self.hd.lock().await;
    let hdpm = guard.parts_mut();
    if hdpm.hb.sorp.len() >= *Usize::from(hdpm.hp.max_concurrent_streams_num()) {
      return Err(crate::Error::http2_go_away_generic(
        Http2Error::ExceedAmountOfActiveConcurrentStreams,
      ));
    }
    let stream_id = *hdpm.last_stream_id;
    let _ = hdpm.hb.scrp.insert(
      stream_id,
      StreamControlRecvParams {
        stream_state: StreamState::Idle,
        windows: Windows::stream(hdpm.hp, hdpm.hps),
      },
    );
    *hdpm.last_stream_id = hdpm.last_stream_id.wrapping_add(U31::TWO);
    drop(guard);
    Ok(ClientStream::new(
      self.hd.clone(),
      _trace_span!("Creating client stream", stream_id = stream_id.u32()),
      stream_id,
    ))
  }
}
