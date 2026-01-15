//! Low-level HTTP/2. You should probably look into higher abstractions like the HTTP server
//! framework.
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
mod initial_server_stream_remote;
mod misc;
mod ping_frame;
mod process_receipt_frame_ty;
mod reader_data;
mod reset_stream_frame;
mod server_stream;
mod settings_frame;
mod stream_receiver;
mod stream_state;
#[cfg(test)]
mod tests;
mod u31;
#[cfg(feature = "web-socket")]
mod web_socket_over_stream;
mod window;
mod window_update_frame;
mod write_functions;
mod writer_data;

use crate::{
  http::{Protocol, ReqResBuffer, Request},
  misc::{ConnectionState, Lease, LeaseMut, SingleTypeStorage, Usize},
  stream::{StreamReader, StreamWriter},
  sync::{Arc, AsyncMutex, AtomicBool, AtomicWaker},
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
pub use server_stream::ServerStream;
#[cfg(feature = "web-socket")]
pub use web_socket_over_stream::WebSocketOverStream;
pub use window::{Window, Windows};

const MAX_BODY_LEN: u32 = max_body_len!();
const MAX_CONCURRENT_STREAMS_NUM: u32 = max_concurrent_streams_num!();
const MAX_FRAME_LEN: u32 = max_frame_len!();
const MAX_FRAME_LEN_LOWER_BOUND: u32 = max_frame_len_lower_bound!();
const MAX_FRAME_LEN_UPPER_BOUND: u32 = max_frame_len_upper_bound!();
const MAX_HEADERS_LEN: u32 = max_headers_len!();
const MAX_HPACK_LEN: u32 = max_hpack_len!();
const MAX_RECV_STREAMS_NUM: u32 = max_recv_streams_num!();
const READ_BUFFER_LEN: u32 = read_buffer_len!();
const PREFACE: &[u8; 24] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

pub(crate) type Scrp = HashMap<u31::U31, stream_receiver::StreamControlRecvParams>;
pub(crate) type Sorp = HashMap<u31::U31, stream_receiver::StreamOverallRecvParams>;

/// Negotiates initial "handshakes" or connections and also manages the creation of streams.
#[derive(Debug)]
pub struct Http2<HB, SW, const IS_CLIENT: bool> {
  inner: Arc<Http2Inner<HB, SW, IS_CLIENT>>,
}

impl<HB, SW, const IS_CLIENT: bool> Http2<HB, SW, IS_CLIENT>
where
  HB: LeaseMut<Http2Buffer>,
  SW: StreamWriter,
{
  /// See [`ConnectionState`].
  #[inline]
  pub fn connection_state(&self) -> ConnectionState {
    misc::connection_state(&self.inner.is_conn_open)
  }

  send_go_away_method!();

  #[cfg(all(feature = "http-client-pool", feature = "tokio"))]
  pub(crate) async fn swap_buffers(&mut self, hb: &mut HB) {
    mem::swap(hb.lease_mut(), self.inner.hd.lock().await.parts_mut().hb);
  }

  async fn manage_initial_params<SR, const HAS_PREFACE: bool>(
    mut hb: HB,
    hp: Http2Params,
    stream_reader: SR,
    mut stream_writer: SW,
  ) -> crate::Result<(impl Future<Output = ()>, Self)>
  where
    SR: StreamReader,
  {
    let is_conn_open = AtomicBool::new(true);
    let sf = hp.to_settings_frame();
    let sf_buffer = &mut [0; 45];
    let sf_bytes = sf.bytes(sf_buffer);
    if hp.initial_window_len() == initial_window_len!() {
      if HAS_PREFACE {
        misc::write_array([PREFACE, sf_bytes], &is_conn_open, &mut stream_writer).await?;
      } else {
        misc::write_array([sf_bytes], &is_conn_open, &mut stream_writer).await?;
      }
    } else {
      let wuf = window_update_frame::WindowUpdateFrame::new(
        hp.initial_window_len().wrapping_sub(initial_window_len!()).into(),
        u31::U31::ZERO,
      )?;
      if HAS_PREFACE {
        let array = [PREFACE, sf_bytes, &wuf.bytes()];
        misc::write_array(array, &is_conn_open, &mut stream_writer).await?;
      } else {
        misc::write_array([sf_bytes, &wuf.bytes()], &is_conn_open, &mut stream_writer).await?;
      }
    }
    hb.lease_mut().hpack_dec.set_max_bytes(hp.max_hpack_len().0);
    hb.lease_mut().hpack_dec.reserve(4, 256)?;
    hb.lease_mut().hpack_enc.set_max_dyn_super_bytes(hp.max_hpack_len().1);
    hb.lease_mut().hpack_enc.reserve(4, 256)?;
    hb.lease_mut().pfb.reserve(*Usize::from(hp.read_buffer_len()))?;
    let rd = reader_data::ReaderData::new(mem::take(&mut hb.lease_mut().pfb), stream_reader);
    let max_frame_len = hp.max_frame_len();
    let wd = writer_data::WriterData::new(stream_writer);
    let inner = Arc::new(Http2Inner {
      hd: AsyncMutex::new(Http2Data::new(hb, hp)),
      is_conn_open,
      read_frame_waker: AtomicWaker::new(),
      wd: AsyncMutex::new(wd),
    });
    Ok((frame_reader::frame_reader(inner.clone(), max_frame_len, rd), Self { inner }))
  }
}

impl<HB, SW> Http2<HB, SW, false>
where
  HB: LeaseMut<Http2Buffer>,
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
      return Err(misc::protocol_err(Http2Error::NoPreface));
    }
    Self::manage_initial_params::<_, false>(hb, hp, stream_reader, stream_writer).await
  }

  /// Awaits for an initial header to create a stream.
  ///
  /// Returns [`None`] if the network connection has been closed, either locally or externally.
  #[inline]
  pub async fn stream<T>(
    &self,
    mut cb: impl FnMut(Request<&mut ReqResBuffer>, Option<Protocol>) -> T,
  ) -> crate::Result<Option<(ServerStream<HB, SW>, T)>> {
    let Self { inner } = self;
    let mut is_registered = false;
    let mut lock_pin = pin!(inner.hd.lock());
    poll_fn(|cx| {
      let mut guard = lock_pin!(cx, inner.hd, lock_pin);
      let hdpm = guard.parts_mut();
      if misc::connection_state(&inner.is_conn_open).is_closed() {
        misc::frame_reader_rslt(hdpm.frame_reader_error)?;
        return Poll::Ready(Ok(None));
      }
      let Some(lss) = hdpm.hb.initial_server_streams_remote.pop_front() else {
        if is_registered {
          misc::frame_reader_rslt(hdpm.frame_reader_error)?;
          return Poll::Ready(Ok(None));
        }
        hdpm.hb.initial_server_streams_local.push_back(cx.waker().clone())?;
        is_registered = true;
        return Poll::Pending;
      };
      let Some(sorp) = hdpm.hb.sorps.get_mut(&lss.stream_id) else {
        // For example, GO_AWAY was sent before receiving a new stream
        misc::frame_reader_rslt(hdpm.frame_reader_error)?;
        return Poll::Ready(Ok(None));
      };
      let (method, protocol, stream_id) = (lss.method, lss.protocol, lss.stream_id);
      Poll::Ready(Ok(Some((
        ServerStream::new(
          inner.clone(),
          method,
          protocol,
          _trace_span!("New server stream", stream_id = %stream_id),
          stream_id,
        ),
        cb(Request::http2(method, &mut sorp.rrb), protocol),
      ))))
    })
    .await
  }
}

impl<HB, SW> Http2<HB, SW, true>
where
  HB: LeaseMut<Http2Buffer>,
  SW: StreamWriter,
{
  /// Tries to connect to a server sending the local parameters.
  #[inline]
  pub async fn connect<SR>(
    mut hb: HB,
    mut hp: Http2Params,
    (stream_reader, stream_writer): (SR, SW),
  ) -> crate::Result<(impl Future<Output = ()>, Self)>
  where
    SR: StreamReader,
  {
    hb.lease_mut().clear();
    hp = hp.set_enable_connect_protocol(false);
    Self::manage_initial_params::<_, true>(hb, hp, stream_reader, stream_writer).await
  }

  /// Opens a local stream.
  #[inline]
  pub async fn stream(&self) -> crate::Result<ClientStream<HB, SW>> {
    let Self { inner } = self;
    let mut hd_guard = inner.hd.lock().await;
    let hdpm = hd_guard.parts_mut();
    if hdpm.hb.sorps.len() >= *Usize::from(hdpm.hp.max_concurrent_streams_num()) {
      drop(hd_guard);
      let err = misc::protocol_err(Http2Error::ExceedAmountOfActiveConcurrentStreams);
      misc::process_higher_operation_err(&err, inner).await;
      return Err(err);
    }
    let stream_id = *hdpm.last_stream_id;
    let span = _trace_span!("New client stream", stream_id = %stream_id);
    drop(hdpm.hb.scrps.insert(
      stream_id,
      stream_receiver::StreamControlRecvParams {
        is_stream_open: true,
        stream_state: stream_state::StreamState::Idle,
        waker: Waker::noop().clone(),
        windows: Windows::initial(hdpm.hp, hdpm.hps),
      },
    ));
    *hdpm.last_stream_id = hdpm.last_stream_id.wrapping_add(u31::U31::TWO);
    drop(hd_guard);
    Ok(ClientStream::new(inner.clone(), span, stream_id))
  }
}

impl<HB, SW, const IS_CLIENT: bool> Lease<Http2<HB, SW, IS_CLIENT>> for Http2<HB, SW, IS_CLIENT> {
  #[inline]
  fn lease(&self) -> &Http2<HB, SW, IS_CLIENT> {
    self
  }
}

impl<HB, SW, const IS_CLIENT: bool> LeaseMut<Http2<HB, SW, IS_CLIENT>>
  for Http2<HB, SW, IS_CLIENT>
{
  #[inline]
  fn lease_mut(&mut self) -> &mut Http2<HB, SW, IS_CLIENT> {
    self
  }
}

impl<HB, SW, const IS_CLIENT: bool> SingleTypeStorage for Http2<HB, SW, IS_CLIENT> {
  type Item = (HB, SW);
}

impl<HB, SW, const IS_CLIENT: bool> Clone for Http2<HB, SW, IS_CLIENT> {
  #[inline]
  fn clone(&self) -> Self {
    Self { inner: self.inner.clone() }
  }
}

#[derive(Debug)]
pub(crate) struct Http2Inner<HB, SW, const IS_CLIENT: bool> {
  pub(crate) hd: AsyncMutex<Http2Data<HB, IS_CLIENT>>,
  pub(crate) is_conn_open: AtomicBool,
  pub(crate) read_frame_waker: AtomicWaker,
  pub(crate) wd: AsyncMutex<writer_data::WriterData<SW>>,
}
