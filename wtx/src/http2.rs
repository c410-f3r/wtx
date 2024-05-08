//! HTTP/2
//!
//! 1. Does not support padded headers when writing.
//! 2. Does not support push promises (Deprecated by the RFC).
//! 3. Does not support prioritization (Deprecated by he RFC).

// Many elements where influenced by the code of the h2 repository (https://github.com/hyperium/h2)
// so thanks to the authors.

#[macro_use]
mod macros;

mod client_stream;
mod continuation_frame;
mod data_frame;
mod error_code;
mod frame_init;
mod go_away_frame;
mod headers_frame;
mod hpack_decoder;
mod hpack_encoder;
mod hpack_header;
mod hpack_static_headers;
mod http2_buffer;
mod http2_data;
mod http2_params;
mod http2_params_send;
mod http2_rslt;
mod huffman;
mod huffman_tables;
mod misc;
mod ping_frame;
mod req_res_buffer;
mod reset_stream_frame;
mod server_stream;
mod settings_frame;
mod stream_state;
#[cfg(test)]
mod tests;
mod u31;
mod uri_buffer;
mod window;
mod window_update_frame;
mod write_stream;

use crate::{
  http2::misc::{apply_initial_params, default_stream_frames, read_frame_until_cb_unknown_id},
  misc::{ConnectionState, LeaseMut, Lock, RefCounter, Stream, Usize},
};
pub use client_stream::ClientStream;
pub(crate) use continuation_frame::ContinuationFrame;
use core::marker::PhantomData;
pub(crate) use data_frame::DataFrame;
pub use error_code::ErrorCode;
pub(crate) use frame_init::{FrameHeaderTy, FrameInit};
pub(crate) use go_away_frame::GoAwayFrame;
pub(crate) use headers_frame::HeadersFrame;
pub(crate) use hpack_decoder::HpackDecoder;
pub(crate) use hpack_encoder::HpackEncoder;
pub(crate) use hpack_header::HpackHeaderBasic;
pub(crate) use hpack_static_headers::{HpackStaticRequestHeaders, HpackStaticResponseHeaders};
pub use http2_buffer::Http2Buffer;
pub(crate) use http2_data::Http2Data;
pub use http2_params::Http2Params;
pub use http2_rslt::Http2Rslt;
pub(crate) use http2_rslt::Http2RsltExt;
pub(crate) use huffman::{huffman_decode, huffman_encode};
pub(crate) use ping_frame::PingFrame;
pub use req_res_buffer::ReqResBuffer;
pub(crate) use reset_stream_frame::ResetStreamFrame;
pub use server_stream::ServerStream;
pub(crate) use settings_frame::SettingsFrame;
pub(crate) use stream_state::StreamState;
use tokio::sync::MutexGuard;
pub(crate) use u31::U31;
pub(crate) use uri_buffer::UriBuffer;
pub(crate) use window::Windows;
pub(crate) use window_update_frame::WindowUpdateFrame;

pub(crate) const MAX_BODY_LEN: u32 = max_body_len!();
pub(crate) const MAX_BUFFERED_FRAMES_NUM: u8 = max_buffered_frames_num!();
pub(crate) const MAX_CACHED_HEADERS_LEN: u32 = max_cached_headers_len!();
pub(crate) const MAX_EXPANDED_HEADERS_LEN: u32 = max_expanded_headers_len!();
pub(crate) const MAX_FRAME_LEN: u32 = max_frame_len!();
pub(crate) const MAX_FRAME_LEN_LOWER_BOUND: u32 = max_frame_len_lower_bound!();
pub(crate) const MAX_FRAME_LEN_UPPER_BOUND: u32 = max_frame_len_upper_bound!();
pub(crate) const MAX_RAPID_RESETS_NUM: u8 = max_rapid_resets_num!();
pub(crate) const MAX_STREAMS_NUM: u32 = max_streams_num!();
pub(crate) const READ_BUFFER_LEN: u32 = read_buffer_len!();

const ACK_MASK: u8 = 0b0000_0001;
const EOH_MASK: u8 = 0b0000_0100;
const EOS_MASK: u8 = 0b0000_0001;
const PAD_MASK: u8 = 0b0000_1000;
const PREFACE: &[u8; 24] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

/// Http2 instance using the mutex from tokio.
#[cfg(feature = "tokio")]
pub type Http2Tokio<HB, S, const IS_CLIENT: bool> =
  Http2<HB, alloc::sync::Arc<tokio::sync::Mutex<Http2Data<HB, S, IS_CLIENT>>>, S, IS_CLIENT>;

/// Negotiates initial "handshakes" or connections and also manages the creation of streams.
#[derive(Debug)]
pub struct Http2<HB, HD, S, const IS_CLIENT: bool> {
  hd: HD,
  phantom: PhantomData<(HB, S)>,
  stream_id: U31,
}

impl<HB, HD, S, const IS_CLIENT: bool> Http2<HB, HD, S, IS_CLIENT>
where
  HB: LeaseMut<Http2Buffer>,
  HD: Lock<Resource = Http2Data<HB, S, IS_CLIENT>>,
  S: Stream,
{
  /// See [ConnectionState].
  #[inline]
  pub async fn connection_state(&self) -> ConnectionState {
    ConnectionState::from(self.hd.lock().await.is_conn_open())
  }

  /// Sends a GOAWAY frame to the peer, which cancels the connection and consequently all ongoing
  /// streams.
  pub async fn send_go_away(self, error_code: ErrorCode) -> crate::Result<()> {
    misc::send_go_away(
      GoAwayFrame::new(error_code, self.stream_id),
      self.hd.lock().await.is_conn_open_and_stream_mut(),
    )
    .await
  }
}

// Remove the `Guard` definition to see a wonderful lifetime error involving `GAT` and
// `Send` bounds that makes anyone feel happy and delightful for uncountable hours.
impl<HB, HD, S> Http2<HB, HD, S, false>
where
  HB: LeaseMut<Http2Buffer>,
  HD: RefCounter,
  for<'guard> HD::Item: Lock<
      Guard<'guard> = MutexGuard<'guard, Http2Data<HB, S, false>>,
      Resource = Http2Data<HB, S, false>,
    > + 'guard,
  S: Stream,
{
  /// Accepts an initial connection sending the local parameters to the remote peer.
  #[inline]
  pub async fn accept(mut hb: HB, hp: Http2Params, mut stream: S) -> crate::Result<Self> {
    hb.lease_mut().clear();
    let mut buffer = [0; 24];
    let _ = stream.read(&mut buffer).await?;
    if &buffer != PREFACE {
      return Err(crate::Error::NoPreface);
    }
    stream.write_all(hp.to_settings_frame().bytes(&mut [0; 45])).await?;
    apply_initial_params(hb.lease_mut(), &hp)?;
    Ok(Self {
      phantom: PhantomData,
      hd: HD::new(HD::Item::new(Http2Data::new(hb, hp, stream))),
      stream_id: U31::ZERO,
    })
  }

  /// Opens a local stream based on initially received headers. See [ServerStream].
  #[inline]
  pub async fn stream(
    &mut self,
    rrb: &mut ReqResBuffer,
  ) -> crate::Result<Http2Rslt<ServerStream<HB, HD, S>>> {
    rrb.clear();
    let mut stream_state = StreamState::Open;
    let mut windows;
    let rfi = hre_to_hr!(
      self.hd,
      |guard| {
        if *guard.streams_num_mut() >= guard.hp().max_streams_num() {
          return Err(crate::Error::ExceedAmountOfActiveConcurrentStreams);
        }
        rrb.headers.set_max_bytes(*Usize::from(guard.hp().max_cached_headers_len().0));
        windows = Windows::stream(guard.hp(), guard.hps());
        guard
          .read_frames_init(
            rrb,
            U31::ZERO,
            &mut stream_state,
            &mut windows,
            |hf| hf.hsreqh().method.ok_or(crate::Error::MissingRequestMethod),
            |data, fi, hp, streams_frames| {
              read_frame_until_cb_unknown_id(data, fi, hp, streams_frames)
            },
          )
          .await?
      },
      |guard, rfi| {
        *guard.streams_num_mut() = guard.streams_num_mut().wrapping_add(1);
        let stream_frame_entry = guard.hb_mut().streams_frames.entry(rfi.stream_id);
        let _ = stream_frame_entry.or_insert_with(default_stream_frames);
      }
    );
    Ok(Http2Rslt::Resource(ServerStream::new(
      self.hd.clone(),
      _trace_span!("Creating server stream", stream_id = rfi.stream_id.u32()),
      rfi,
      stream_state,
      windows,
    )))
  }
}

impl<HB, HD, S> Http2<HB, HD, S, true>
where
  HB: LeaseMut<Http2Buffer>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, S, true>>,
  S: Stream,
{
  /// Tries to connect to a server sending the local parameters.
  #[inline]
  pub async fn connect(mut hb: HB, hp: Http2Params, mut stream: S) -> crate::Result<Self> {
    hb.lease_mut().clear();
    stream.write_all(PREFACE).await?;
    stream.write_all(hp.to_settings_frame().bytes(&mut [0; 45])).await?;
    apply_initial_params(hb.lease_mut(), &hp)?;
    Ok(Self {
      phantom: PhantomData,
      hd: HD::new(HD::Item::new(Http2Data::new(hb, hp, stream))),
      stream_id: U31::ONE,
    })
  }

  /// Opens a local stream. See [ClientStream].
  pub async fn stream(&mut self) -> crate::Result<ClientStream<HB, HD, S>> {
    let mut guard = self.hd.lock().await;
    if *guard.streams_num_mut() >= guard.send_params_mut().max_streams_num {
      return Err(crate::Error::ExceedAmountOfActiveConcurrentStreams);
    }
    let windows = Windows::stream(guard.hp(), guard.hps());
    *guard.streams_num_mut() = guard.streams_num_mut().wrapping_add(1);
    let stream_id = self.stream_id;
    self.stream_id = self.stream_id.wrapping_add(U31::TWO);
    let _ = guard.hb_mut().streams_frames.entry(stream_id).or_insert_with(default_stream_frames);
    Ok(ClientStream::idle(
      self.hd.clone(),
      _trace_span!("Creating client stream", stream_id = stream_id.u32()),
      stream_id,
      windows,
    ))
  }
}
