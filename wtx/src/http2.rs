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
mod http2_buffer;
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
  http::{HttpError, ReqResBuffer, StatusCode},
  http2::misc::{
    process_higher_operation_err, protocol_err, read_header_and_continuations,
    server_header_stream_state,
  },
  misc::{ConnectionState, LeaseMut, Lock, RefCounter, Stream, Usize, _Span},
};
pub use client_stream::ClientStream;
pub(crate) use common_flags::CommonFlags;
pub(crate) use continuation_frame::ContinuationFrame;
use core::{mem, time::Duration};
pub(crate) use data_frame::DataFrame;
pub(crate) use frame_init::{FrameInit, FrameInitTy};
pub(crate) use go_away_frame::GoAwayFrame;
use hashbrown::HashMap;
pub(crate) use headers_frame::HeadersFrame;
pub(crate) use hpack_decoder::HpackDecoder;
pub(crate) use hpack_encoder::HpackEncoder;
pub(crate) use hpack_header::HpackHeaderBasic;
pub(crate) use hpack_static_headers::{HpackStaticRequestHeaders, HpackStaticResponseHeaders};
pub use http2_buffer::Http2Buffer;
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
pub type Http2Tokio<HB, RRB, S, const IS_CLIENT: bool> =
  Http2<Http2DataTokio<HB, RRB, S, IS_CLIENT>, IS_CLIENT>;
/// Http2Data instance using the mutex from tokio.
#[cfg(feature = "tokio")]
pub type Http2DataTokio<HB, RRB, S, const IS_CLIENT: bool> =
  alloc::sync::Arc<tokio::sync::Mutex<Http2Data<HB, RRB, S, IS_CLIENT>>>;

pub(crate) type Sorp<RRB> = HashMap<U31, StreamOverallRecvParams<RRB>>;
pub(crate) type Scrp = HashMap<U31, StreamControlRecvParams>;

/// Negotiates initial "handshakes" or connections and also manages the creation of streams.
#[derive(Debug)]
pub struct Http2<HD, const IS_CLIENT: bool> {
  hd: HD,
}

impl<HB, HD, RRB, S, const IS_CLIENT: bool> Http2<HD, IS_CLIENT>
where
  HB: LeaseMut<Http2Buffer<RRB>>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, RRB, S, IS_CLIENT>>,
  RRB: LeaseMut<ReqResBuffer>,
  S: Stream,
{
  /// See [ConnectionState].
  #[inline]
  pub async fn connection_state(&self) -> ConnectionState {
    ConnectionState::from(self.hd.lock().await.is_conn_open())
  }

  /// Sends a GOAWAY frame to the peer, which cancels the connection and consequently all ongoing
  /// streams.
  #[inline]
  pub async fn send_go_away(self, error_code: Http2ErrorCode) {
    let mut guard = self.hd.lock().await;
    let hdpm = guard.parts_mut();
    misc::send_go_away(error_code, hdpm.is_conn_open, *hdpm.last_stream_id, hdpm.stream).await;
  }

  #[inline]
  pub(crate) async fn _swap_buffers(&mut self, hb: &mut HB) {
    mem::swap(hb.lease_mut(), self.hd.lock().await.parts_mut().hb);
  }

  #[inline]
  async fn apply_initial_params(
    hb: &mut Http2Buffer<RRB>,
    has_preface: bool,
    hp: &Http2Params,
    stream: &mut S,
  ) -> crate::Result<()> {
    let sf = hp.to_settings_frame();
    if hp.initial_window_len() != initial_window_len!() {
      let wuf = WindowUpdateFrame::new(
        hp.initial_window_len().wrapping_sub(initial_window_len!()).into(),
        U31::ZERO,
      )?;
      if has_preface {
        stream.write_all_vectored(&[PREFACE, sf.bytes(&mut [0; 45]), &wuf.bytes()]).await?;
      } else {
        stream.write_all_vectored(&[sf.bytes(&mut [0; 45]), &wuf.bytes()]).await?;
      }
    } else {
      if has_preface {
        stream.write_all_vectored(&[PREFACE, sf.bytes(&mut [0; 45])]).await?;
      } else {
        stream.write_all(sf.bytes(&mut [0; 45])).await?;
      }
    }
    hb.hpack_dec.set_max_bytes(hp.max_hpack_len().0);
    hb.hpack_enc.set_max_dyn_super_bytes(hp.max_hpack_len().1);
    hb.pfb._expand_buffer(*Usize::from(hp.read_buffer_len()))?;
    Ok(())
  }
}

impl<HB, HD, RRB, S> Http2<HD, false>
where
  HB: LeaseMut<Http2Buffer<RRB>>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, RRB, S, false>>,
  RRB: LeaseMut<ReqResBuffer>,
  S: Stream,
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
    Self::apply_initial_params(hb.lease_mut(), false, &hp, &mut stream).await?;
    Ok(Self { hd: HD::new(HD::Item::new(Http2Data::new(hb, hp, stream))) })
  }

  /// Awaits for an initial header to create a stream.
  #[inline]
  pub async fn stream(&mut self, mut rrb: RRB) -> crate::Result<ServerStream<HD>> {
    rrb.lease_mut().clear();
    process_higher_operation!(
      &self.hd,
      |guard| {
        let rslt = 'rslt: {
          let hdpm = guard.parts_mut();
          let Some(fi) = hdpm.hb.initial_server_header.take() else {
            break 'rslt Ok(None);
          };
          rrb.lease_mut().headers_mut().set_max_bytes(*Usize::from(hdpm.hp.max_headers_len()));
          let fut = read_header_and_continuations::<_, _, false, false>(
            fi,
            hdpm.hp,
            &mut hdpm.hb.hpack_dec,
            hdpm.is_conn_open,
            &mut hdpm.hb.pfb,
            rrb.lease_mut(),
            hdpm.stream,
            &mut hdpm.hb.uri_buffer,
            |hf| hf.hsreqh().method.ok_or_else(|| HttpError::MissingRequestMethod.into()),
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
            rrb,
            span: _Span::_none(),
            status_code: StatusCode::Ok,
            stream_state: server_header_stream_state(has_eos),
            windows: Windows::initial(hdpm.hp, hdpm.hps),
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

impl<HB, HD, RRB, S> Http2<HD, true>
where
  HB: LeaseMut<Http2Buffer<RRB>>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, RRB, S, true>>,
  RRB: LeaseMut<ReqResBuffer>,
  S: Stream,
{
  /// Tries to connect to a server sending the local parameters.
  #[inline]
  pub async fn connect(mut hb: HB, hp: Http2Params, mut stream: S) -> crate::Result<Self> {
    hb.lease_mut().clear();
    Self::apply_initial_params(hb.lease_mut(), true, &hp, &mut stream).await?;
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
        windows: Windows::initial(hdpm.hp, hdpm.hps),
      },
    ));
    *hdpm.last_stream_id = hdpm.last_stream_id.wrapping_add(U31::TWO);
    drop(guard);
    Ok(ClientStream::new(self.hd.clone(), span, stream_id))
  }
}

impl<HD, const IS_CLIENT: bool> Clone for Http2<HD, IS_CLIENT>
where
  HD: RefCounter,
{
  #[inline]
  fn clone(&self) -> Self {
    Self { hd: self.hd.clone() }
  }
}
