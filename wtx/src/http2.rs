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
mod common_flags;
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
  http::StatusCode,
  http2::misc::{
    apply_initial_params, process_higher_operation_err, protocol_err,
    read_header_and_continuations, server_header_stream_state,
  },
  misc::{ConnectionState, LeaseMut, Lock, RefCounter, Stream, Usize, _Span},
};
pub use buffers::{Http2Buffer, ReqResBuffer, StreamBuffer};
pub use client_stream::ClientStream;
pub(crate) use common_flags::CommonFlags;
pub(crate) use continuation_frame::ContinuationFrame;
use core::time::Duration;
pub(crate) use data_frame::DataFrame;
pub(crate) use frame_init::{FrameInit, FrameInitTy};
pub(crate) use go_away_frame::GoAwayFrame;
use hashbrown::HashMap;
pub(crate) use headers_frame::HeadersFrame;
pub(crate) use hpack_decoder::HpackDecoder;
pub(crate) use hpack_encoder::HpackEncoder;
pub(crate) use hpack_header::HpackHeaderBasic;
pub(crate) use hpack_static_headers::{HpackStaticRequestHeaders, HpackStaticResponseHeaders};
pub use http2_data::Http2Data;
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

const MAX_FINAL_DURATION: Duration = Duration::from_millis(300);
const MAX_FINAL_FETCHES: u8 = 32;
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
    stream.read_exact(&mut buffer).await?;
    if &buffer != PREFACE {
      misc::send_go_away(Http2ErrorCode::ProtocolError, &mut true, U31::ZERO, &mut stream).await;
      return Err(protocol_err(Http2Error::NoPreface));
    }
    stream.write_all(hp.to_settings_frame().bytes(&mut [0; 45])).await?;
    apply_initial_params(hb.lease_mut(), &hp)?;
    Ok(Self { hd: HD::new(HD::Item::new(Http2Data::new(hb, hp, stream))) })
  }

  /// Awaits for an initial header to create a stream.
  #[inline]
  pub async fn stream(&mut self, mut sb: SB) -> crate::Result<ServerStream<HD>> {
    sb.lease_mut().clear();
    process_higher_operation!(
      &self.hd,
      |guard| {
        let rslt = 'rslt: {
          let hdpm = guard.parts_mut();
          let Some(fi) = hdpm.hb.initial_server_header.take() else {
            break 'rslt Ok(None);
          };
          sb.lease_mut().rrb.headers.set_max_bytes(*Usize::from(hdpm.hp.max_headers_len()));
          let fut = read_header_and_continuations::<_, _, false, false>(
            fi,
            hdpm.hp,
            &mut hdpm.hb.hpack_dec,
            hdpm.is_conn_open,
            &mut hdpm.hb.pfb,
            &mut sb.lease_mut().rrb,
            hdpm.stream,
            &mut hdpm.hb.uri_buffer,
            |hf| hf.hsreqh().method.ok_or(crate::Error::HTTP_MissingRequestMethod),
          );
          match fut.await {
            Err(err) => break 'rslt Err(err),
            Ok(elem) => Ok(Some((elem.0, fi, elem.1, elem.2))),
          }
        };
        rslt
      },
      |guard, elem| {
        let (content_length_idx, fi, has_eos, method) = elem;
        let hdpm = guard.parts_mut();
        drop(hdpm.hb.sorp.insert(
          fi.stream_id,
          StreamOverallRecvParams {
            body_len: 0,
            content_length_idx,
            has_initial_header: true,
            sb,
            span: _Span::_none(),
            status_code: StatusCode::Ok,
            stream_state: server_header_stream_state(has_eos),
            windows: Windows::stream(hdpm.hp, hdpm.hps),
          },
        ));
        Ok(ServerStream::new(
          self.hd.clone(),
          method,
          _trace_span!("Creating server stream", stream_id = fi.stream_id.u32()),
          fi.stream_id,
        ))
      }
    )
  }
}

impl<HB, HD, S, SB> Http2<HD, true>
where
  HB: LeaseMut<Http2Buffer<SB>>,
  HD: RefCounter,
  for<'guard> HD::Item: Lock<
      Guard<'guard> = MutexGuard<'guard, Http2Data<HB, S, SB, true>>,
      Resource = Http2Data<HB, S, SB, true>,
    > + 'guard,
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
      drop(guard);
      let err = Http2Error::ExceedAmountOfActiveConcurrentStreams;
      return Err(process_higher_operation_err(protocol_err(err), &self.hd).await);
    }
    let stream_id = *hdpm.last_stream_id;
    let span = _trace_span!("Creating client stream", stream_id = stream_id.u32());
    drop(hdpm.hb.scrp.insert(
      stream_id,
      StreamControlRecvParams {
        span: span.clone(),
        stream_state: StreamState::Idle,
        windows: Windows::stream(hdpm.hp, hdpm.hps),
      },
    ));
    *hdpm.last_stream_id = hdpm.last_stream_id.wrapping_add(U31::TWO);
    drop(guard);
    Ok(ClientStream::new(self.hd.clone(), span, stream_id))
  }
}
