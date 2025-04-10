//! HTTP/2
//!
//! 1. Does not support padded headers when writing.
//! 2. Does not support push promises (Deprecated by major third-parties).
//! 3. Does not support prioritization (Deprecated by the RFC).

#[macro_use]
mod macros;

mod client_stream;
mod common_flags;
mod common_stream;
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
mod http2_status;
mod huffman;
mod huffman_tables;
mod index_map;
mod initial_server_header;
mod misc;
mod ping_frame;
mod process_receipt_frame_ty;
mod reset_stream_frame;
mod send_data_mode;
mod send_msg;
mod server_stream;
mod settings_frame;
mod stream_receiver;
mod stream_state;
#[cfg(all(feature = "_async-tests", test))]
mod tests;
mod u31;
mod uri_buffer;
#[cfg(feature = "web-socket")]
mod web_socket_over_stream;
mod window;
mod window_update_frame;

pub use crate::sync::Ordering;
use crate::{
  http::{Method, Protocol, ReqResBuffer, Request},
  http2::misc::{
    frame_reader_rslt, manage_initial_stream_receiving, process_higher_operation_err, protocol_err,
    sorp_mut, write_array,
  },
  misc::{
    ConnectionState, Either, Lease, LeaseMut, Lock, RefCounter, SingleTypeStorage, StreamReader,
    StreamWriter, Usize, net::PartitionedFilledBuffer,
  },
  sync::{Arc, AtomicBool, AtomicWaker},
};
pub use client_stream::ClientStream;
pub use common_stream::CommonStream;
use core::{
  future::poll_fn,
  mem,
  pin::pin,
  task::{Poll, Waker},
};
use hashbrown::HashMap;
pub use http2_buffer::Http2Buffer;
pub use http2_data::Http2Data;
pub use http2_error::Http2Error;
pub use http2_error_code::Http2ErrorCode;
pub use http2_params::Http2Params;
pub use http2_status::{Http2RecvStatus, Http2SendStatus};
pub use send_data_mode::{SendDataMode, SendDataModeBytes};
pub use server_stream::ServerStream;
#[cfg(feature = "web-socket")]
pub use web_socket_over_stream::WebSocketOverStream;
pub use window::{Window, Windows};

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
pub type Http2Tokio<HB, SW, const IS_CLIENT: bool> =
  Http2<Http2DataTokio<HB, SW, IS_CLIENT>, IS_CLIENT>;
/// [`Http2Data`] instance using the mutex from tokio.
#[cfg(feature = "tokio")]
pub type Http2DataTokio<HB, SW, const IS_CLIENT: bool> =
  Arc<tokio::sync::Mutex<Http2Data<HB, SW, IS_CLIENT>>>;

pub(crate) type Scrp = HashMap<u31::U31, stream_receiver::StreamControlRecvParams>;
pub(crate) type Sorp = HashMap<u31::U31, stream_receiver::StreamOverallRecvParams>;

/// Negotiates initial "handshakes" or connections and also manages the creation of streams.
#[derive(Debug)]
pub struct Http2<HD, const IS_CLIENT: bool> {
  hd: HD,
  is_conn_open: Arc<AtomicBool>,
  ish_id: u32,
}

impl<HB, HD, SW, const IS_CLIENT: bool> Http2<HD, IS_CLIENT>
where
  HB: LeaseMut<Http2Buffer>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, SW, IS_CLIENT>>,
  SW: StreamWriter,
{
  /// See [`ConnectionState`].
  #[inline]
  pub fn connection_state(&self) -> ConnectionState {
    ConnectionState::from(self.is_conn_open.load(Ordering::Relaxed))
  }

  send_go_away_method!();

  #[inline]
  pub(crate) async fn _swap_buffers(&mut self, hb: &mut HB) {
    mem::swap(hb.lease_mut(), self.hd.lock().await.parts_mut().hb);
  }

  #[inline]
  async fn manage_initial_params<const HAS_PREFACE: bool>(
    hb: &mut Http2Buffer,
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
      let wuf = window_update_frame::WindowUpdateFrame::new(
        hp.initial_window_len().wrapping_sub(initial_window_len!()).into(),
        u31::U31::ZERO,
      )?;
      if HAS_PREFACE {
        write_array([PREFACE, sf_bytes, &wuf.bytes()], &hb.is_conn_open, stream_writer).await?;
      } else {
        write_array([sf_bytes, &wuf.bytes()], &hb.is_conn_open, stream_writer).await?;
      }
    }
    hb.hpack_dec.set_max_bytes(hp.max_hpack_len().0);
    hb.hpack_dec.reserve(4, 256)?;
    hb.hpack_enc.set_max_dyn_super_bytes(hp.max_hpack_len().1);
    hb.hpack_enc.reserve(4, 256)?;
    hb.pfb._reserve(*Usize::from(hp.read_buffer_len()))?;
    Ok((
      Arc::clone(&hb.is_conn_open),
      hp.max_frame_len(),
      mem::take(&mut hb.pfb),
      Arc::clone(&hb.read_frame_waker),
    ))
  }
}

impl<HB, HD, SW> Http2<HD, false>
where
  HB: LeaseMut<Http2Buffer>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, SW, false>>,
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
        .write_all(
          &go_away_frame::GoAwayFrame::new(Http2ErrorCode::ProtocolError, u31::U31::ZERO).bytes(),
        )
        .await;
      return Err(protocol_err(Http2Error::NoPreface));
    }
    let (is_conn_open, max_frame_len, pfb, read_frame_waker) =
      Self::manage_initial_params::<false>(hb.lease_mut(), &hp, &mut stream_writer).await?;
    let hd = HD::new(HD::Item::new(Http2Data::new(hb, hp, stream_writer)));
    let this = Self { hd: hd.clone(), is_conn_open: Arc::clone(&is_conn_open), ish_id: 0 };
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
  pub async fn stream<T>(
    &mut self,
    rrb: ReqResBuffer,
    cb: impl FnOnce(Request<&mut ReqResBuffer>, Option<Protocol>) -> T,
  ) -> crate::Result<Either<ReqResBuffer, (ServerStream<HD>, T)>> {
    let Self { hd, is_conn_open, ish_id } = self;
    let curr_ish_id = *ish_id;
    *ish_id = ish_id.wrapping_add(1);
    let rrb_opt = &mut Some(rrb);
    let mut lock_pin = pin!(hd.lock());
    let rslt = poll_fn(|cx| {
      let mut guard = lock_pin!(cx, hd, lock_pin);
      let hdpm = guard.parts_mut();
      if let Some(mut this_rrb) = rrb_opt.take() {
        if !manage_initial_stream_receiving(is_conn_open, &mut this_rrb) {
          return Poll::Ready(Ok(Either::Left((
            this_rrb,
            frame_reader_rslt(hdpm.frame_reader_error),
          ))));
        }
        drop(hdpm.hb.initial_server_headers.push_back(
          curr_ish_id,
          initial_server_header::InitialServerHeader {
            method: Method::Get,
            protocol: None,
            rrb: this_rrb,
            stream_id: u31::U31::ZERO,
            waker: cx.waker().clone(),
          },
        ));
        Poll::Pending
      } else {
        let Some(ish) = hdpm.hb.initial_server_headers.remove(&curr_ish_id) else {
          return Poll::Ready(Err(protocol_err(Http2Error::UnknownInitialServerHeaderId)));
        };
        hdpm.hb.initial_server_headers.decrease_cursor();
        if !is_conn_open.load(Ordering::Relaxed) {
          let this_rrb = if ish.stream_id.is_zero() {
            ish.rrb
          } else {
            mem::take(&mut sorp_mut(&mut hdpm.hb.sorp, ish.stream_id)?.rrb)
          };
          return Poll::Ready(Ok(Either::Left((
            this_rrb,
            frame_reader_rslt(hdpm.frame_reader_error),
          ))));
        }
        Poll::Ready(Ok(Either::Right((ish.method, ish.protocol, ish.stream_id, guard))))
      }
    })
    .await;
    match rslt? {
      Either::Left(elem) => {
        elem.1?;
        Ok(Either::Left(elem.0))
      }
      Either::Right((method, protocol, stream_id, mut guard)) => {
        let sorp = sorp_mut(&mut guard.parts_mut().hb.sorp, stream_id)?;
        let elem_cb = cb(Request::http2(method, &mut sorp.rrb), protocol);
        drop(guard);
        Ok(Either::Right((
          ServerStream::new(
            hd.clone(),
            Arc::clone(is_conn_open),
            method,
            protocol,
            _trace_span!("New server stream", stream_id = %stream_id),
            stream_id,
          ),
          elem_cb,
        )))
      }
    }
  }
}

impl<HB, HD, SW> Http2<HD, true>
where
  HB: LeaseMut<Http2Buffer>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, SW, true>>,
  SW: StreamWriter,
{
  /// Tries to connect to a server sending the local parameters.
  #[inline]
  pub async fn connect<SR>(
    mut hb: HB,
    mut hp: Http2Params,
    (stream_reader, mut stream_writer): (SR, SW),
  ) -> crate::Result<(impl Future<Output = ()>, Self)>
  where
    SR: StreamReader,
  {
    hb.lease_mut().clear();
    hp = hp.set_enable_connect_protocol(false);
    let (is_conn_open, max_frame_len, pfb, read_frame_waker) =
      Self::manage_initial_params::<true>(hb.lease_mut(), &hp, &mut stream_writer).await?;
    let hd = HD::new(HD::Item::new(Http2Data::new(hb, hp, stream_writer)));
    let this = Self { hd: hd.clone(), is_conn_open: Arc::clone(&is_conn_open), ish_id: 0 };
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
      stream_receiver::StreamControlRecvParams {
        is_stream_open: true,
        stream_state: stream_state::StreamState::Idle,
        waker: Waker::noop().clone(),
        windows: Windows::initial(hdpm.hp, hdpm.hps),
      },
    ));
    *hdpm.last_stream_id = hdpm.last_stream_id.wrapping_add(u31::U31::TWO);
    drop(guard);
    Ok(ClientStream::new(self.hd.clone(), Arc::clone(&self.is_conn_open), span, stream_id))
  }
}

impl<HD, const IS_CLIENT: bool> Lease<Http2<HD, IS_CLIENT>> for Http2<HD, IS_CLIENT> {
  #[inline]
  fn lease(&self) -> &Http2<HD, IS_CLIENT> {
    self
  }
}

impl<HD, const IS_CLIENT: bool> LeaseMut<Http2<HD, IS_CLIENT>> for Http2<HD, IS_CLIENT> {
  #[inline]
  fn lease_mut(&mut self) -> &mut Http2<HD, IS_CLIENT> {
    self
  }
}

impl<HD, const IS_CLIENT: bool> SingleTypeStorage for Http2<HD, IS_CLIENT> {
  type Item = HD;
}

impl<HD, const IS_CLIENT: bool> Clone for Http2<HD, IS_CLIENT>
where
  HD: RefCounter,
{
  #[inline]
  fn clone(&self) -> Self {
    Self { hd: self.hd.clone(), is_conn_open: Arc::clone(&self.is_conn_open), ish_id: self.ish_id }
  }
}
