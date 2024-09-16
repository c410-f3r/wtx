//! HTTP/2
//!
//! 1. Does not support padded headers when writing.
//! 2. Does not support push promises (Deprecated by the RFC).
//! 3. Does not support prioritization (Deprecated by the RFC).

#[macro_use]
mod macros;

mod client_stream;
mod common_flags;
mod continuation_frame;
mod data_frame;
mod frame_init;
mod frame_reader;
mod go_away_frame;
mod headers_frame;
mod hpack_decoder;
mod hpack_encoder;
mod hpack_header;
mod hpack_headers;
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
  http::ReqResBuffer,
  http2::misc::{
    frame_reader_rslt, manage_initial_stream_receiving, process_higher_operation_err, protocol_err,
    write_array,
  },
  misc::{
    AtomicWaker, ConnectionState, Either, LeaseMut, Lock, PartitionedFilledBuffer, RefCounter,
    StreamReader, StreamWriter, Usize,
  },
};
use alloc::sync::Arc;
pub use client_stream::ClientStream;
pub(crate) use common_flags::CommonFlags;
pub(crate) use continuation_frame::ContinuationFrame;
use core::{
  future::{poll_fn, Future},
  mem,
  pin::pin,
  sync::atomic::{AtomicBool, Ordering},
  task::{Poll, Waker},
};
pub(crate) use data_frame::DataFrame;
pub(crate) use frame_init::{FrameInit, FrameInitTy};
pub(crate) use go_away_frame::GoAwayFrame;
use hashbrown::HashMap;
pub(crate) use headers_frame::HeadersFrame;
pub(crate) use hpack_decoder::HpackDecoder;
pub(crate) use hpack_encoder::HpackEncoder;
pub(crate) use hpack_header::HpackHeaderBasic;
pub(crate) use hpack_headers::HpackHeaders;
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

const PREFACE: &[u8; 24] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

/// [`Http2`] instance using the mutex from tokio.
#[cfg(feature = "tokio")]
pub type Http2Tokio<HB, RRB, SW, const IS_CLIENT: bool> =
  Http2<Http2DataTokio<HB, RRB, SW, IS_CLIENT>, IS_CLIENT>;
/// [`Http2Data`] instance using the mutex from tokio.
#[cfg(feature = "tokio")]
pub type Http2DataTokio<HB, RRB, SW, const IS_CLIENT: bool> =
  Arc<tokio::sync::Mutex<Http2Data<HB, RRB, SW, IS_CLIENT>>>;

pub(crate) type Scrp = HashMap<U31, StreamControlRecvParams>;
pub(crate) type Sorp<RRB> = HashMap<U31, StreamOverallRecvParams<RRB>>;

/// Negotiates initial "handshakes" or connections and also manages the creation of streams.
#[derive(Debug)]
pub struct Http2<HD, const IS_CLIENT: bool> {
  hd: HD,
  is_conn_open: Arc<AtomicBool>,
}

impl<HB, HD, RRB, SW, const IS_CLIENT: bool> Http2<HD, IS_CLIENT>
where
  HB: LeaseMut<Http2Buffer<RRB>>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, RRB, SW, IS_CLIENT>>,
  RRB: LeaseMut<ReqResBuffer>,
  SW: StreamWriter,
{
  /// See [`ConnectionState`].
  #[inline]
  pub fn connection_state(&self) -> ConnectionState {
    ConnectionState::from(self.is_conn_open.load(Ordering::Relaxed))
  }

  /// Sends a GOAWAY frame to the peer, which cancels the connection and consequently all ongoing
  /// streams.
  #[inline]
  pub async fn send_go_away(self, error_code: Http2ErrorCode) {
    misc::send_go_away(error_code, &mut self.hd.lock().await.parts_mut()).await;
  }

  #[inline]
  pub(crate) async fn _swap_buffers(&mut self, hb: &mut HB) {
    mem::swap(hb.lease_mut(), self.hd.lock().await.parts_mut().hb);
  }

  #[inline]
  async fn manage_initial_params<const HAS_PREFACE: bool>(
    hb: &mut Http2Buffer<RRB>,
    hp: &Http2Params,
    stream_writer: &mut SW,
  ) -> crate::Result<(Arc<AtomicBool>, u32, PartitionedFilledBuffer, Arc<AtomicWaker>)> {
    hb.is_conn_open.store(true, Ordering::Relaxed);
    let sf = hp.to_settings_frame();
    let sf_buffer = &mut [0; 45];
    let sf_bytes = sf.bytes(sf_buffer);
    if hp.initial_window_len() == initial_window_len!() {
      if HAS_PREFACE {
        write_array([PREFACE, sf_bytes], &hb.is_conn_open, stream_writer).await?;
      } else {
        write_array([sf_bytes], &hb.is_conn_open, stream_writer).await?;
      }
    } else {
      let wuf = WindowUpdateFrame::new(
        hp.initial_window_len().wrapping_sub(initial_window_len!()).into(),
        U31::ZERO,
      )?;
      if HAS_PREFACE {
        write_array([PREFACE, sf_bytes, &wuf.bytes()], &hb.is_conn_open, stream_writer).await?;
      } else {
        write_array([sf_bytes, &wuf.bytes()], &hb.is_conn_open, stream_writer).await?;
      }
    }
    hb.hpack_dec.set_max_bytes(hp.max_hpack_len().0);
    hb.hpack_enc.set_max_dyn_super_bytes(hp.max_hpack_len().1);
    hb.pfb._expand_buffer(*Usize::from(hp.read_buffer_len()))?;
    Ok((
      Arc::clone(&hb.is_conn_open),
      hp.max_frame_len(),
      mem::take(&mut hb.pfb),
      Arc::clone(&hb.read_frame_waker),
    ))
  }
}

impl<HB, HD, RRB, SW> Http2<HD, false>
where
  HB: LeaseMut<Http2Buffer<RRB>>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, RRB, SW, false>>,
  RRB: LeaseMut<ReqResBuffer>,
  SW: StreamWriter,
{
  /// Accepts an initial connection sending the local parameters to the remote peer.
  #[inline]
  pub async fn accept<SR>(
    mut hb: HB,
    hp: Http2Params,
    (mut stream_reader, mut stream_writer): (SR, SW),
  ) -> crate::Result<(impl Future<Output = ()>, Self)>
  where
    SR: StreamReader,
  {
    hb.lease_mut().clear();
    let mut buffer = [0; 24];
    let _ = stream_reader.read(&mut buffer).await?;
    if &buffer != PREFACE {
      let _rslt = stream_writer
        .write_all(&GoAwayFrame::new(Http2ErrorCode::ProtocolError, U31::ZERO).bytes())
        .await;
      return Err(protocol_err(Http2Error::NoPreface));
    }
    let (is_conn_open, max_frame_len, pfb, read_frame_waker) =
      Self::manage_initial_params::<false>(hb.lease_mut(), &hp, &mut stream_writer).await?;
    let hd = HD::new(HD::Item::new(Http2Data::new(hb, hp, stream_writer)));
    let this = Self { hd: hd.clone(), is_conn_open: Arc::clone(&is_conn_open) };
    Ok((
      frame_reader::frame_reader(
        hd,
        is_conn_open,
        max_frame_len,
        pfb,
        read_frame_waker,
        stream_reader,
      ),
      this,
    ))
  }

  /// Awaits for an initial header to create a stream.
  ///
  /// Returns [`Either::Left`] if the network connection has been closed, either locally
  /// or externally.
  #[inline]
  pub async fn stream(&mut self, rrb: RRB) -> crate::Result<Either<Option<RRB>, ServerStream<HD>>> {
    let Self { hd, is_conn_open } = self;
    let rrb_opt = &mut Some(rrb);
    let mut lock_pin = pin!(hd.lock());
    match poll_fn(|cx| {
      let mut lock = lock_pin!(cx, hd, lock_pin);
      let hdpm = lock.parts_mut();
      if let Some(mut elem) = rrb_opt.take() {
        if !manage_initial_stream_receiving(is_conn_open, &mut elem) {
          let rslt = frame_reader_rslt(hdpm.frame_reader_error);
          return Poll::Ready(Either::Left((Some(elem), rslt)));
        }
        hdpm.hb.initial_server_header_buffers.push_back((elem, cx.waker().clone()));
        Poll::Pending
      } else {
        if !is_conn_open.load(Ordering::Relaxed) {
          return Poll::Ready(Either::Left((
            hdpm.hb.initial_server_header_buffers.pop_front().map(|el| el.0),
            frame_reader_rslt(hdpm.frame_reader_error),
          )));
        }
        let Some((method, stream_id)) = hdpm.hb.initial_server_header_params.pop_front() else {
          return Poll::Pending;
        };
        Poll::Ready(Either::Right((method, stream_id)))
      }
    })
    .await
    {
      Either::Left(elem) => {
        elem.1?;
        Ok(Either::Left(elem.0))
      }
      Either::Right((method, stream_id)) => Ok(Either::Right(ServerStream::new(
        hd.clone(),
        Arc::clone(is_conn_open),
        method,
        _trace_span!("New server stream", stream_id = %stream_id),
        stream_id,
      ))),
    }
  }
}

impl<HB, HD, RRB, SW> Http2<HD, true>
where
  HB: LeaseMut<Http2Buffer<RRB>>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, RRB, SW, true>>,
  RRB: LeaseMut<ReqResBuffer>,
  SW: StreamWriter,
{
  /// Tries to connect to a server sending the local parameters.
  #[inline]
  pub async fn connect<SR>(
    mut hb: HB,
    hp: Http2Params,
    (stream_reader, mut stream_writer): (SR, SW),
  ) -> crate::Result<(impl Future<Output = ()>, Self)>
  where
    SR: StreamReader,
  {
    hb.lease_mut().clear();
    let (is_conn_open, max_frame_len, pfb, read_frame_waker) =
      Self::manage_initial_params::<true>(hb.lease_mut(), &hp, &mut stream_writer).await?;
    let hd = HD::new(HD::Item::new(Http2Data::new(hb, hp, stream_writer)));
    let this = Self { hd: hd.clone(), is_conn_open: Arc::clone(&is_conn_open) };
    Ok((
      frame_reader::frame_reader(
        hd,
        is_conn_open,
        max_frame_len,
        pfb,
        read_frame_waker,
        stream_reader,
      ),
      this,
    ))
  }

  /// Opens a local stream.
  #[inline]
  pub async fn stream(&mut self) -> crate::Result<ClientStream<HD>> {
    let mut guard = self.hd.lock().await;
    let hdpm = guard.parts_mut();
    if hdpm.hb.sorp.len() >= *Usize::from(hdpm.hp.max_concurrent_streams_num()) {
      drop(guard);
      let err = protocol_err(Http2Error::ExceedAmountOfActiveConcurrentStreams);
      process_higher_operation_err(&err, &self.hd).await;
      return Err(err);
    }
    let stream_id = *hdpm.last_stream_id;
    let span = _trace_span!("New client stream", stream_id = %stream_id);
    drop(hdpm.hb.scrp.insert(
      stream_id,
      StreamControlRecvParams {
        is_stream_open: true,
        stream_state: StreamState::Idle,
        waker: Waker::noop().clone(),
        windows: Windows::initial(hdpm.hp, hdpm.hps),
      },
    ));
    *hdpm.last_stream_id = hdpm.last_stream_id.wrapping_add(U31::TWO);
    drop(guard);
    Ok(ClientStream::new(self.hd.clone(), Arc::clone(&self.is_conn_open), span, stream_id))
  }
}

impl<HD, const IS_CLIENT: bool> Clone for Http2<HD, IS_CLIENT>
where
  HD: RefCounter,
{
  #[inline]
  fn clone(&self) -> Self {
    Self { hd: self.hd.clone(), is_conn_open: Arc::clone(&self.is_conn_open) }
  }
}
