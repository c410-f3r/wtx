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
mod http2_status;
mod http_send_params;
mod huffman;
mod huffman_tables;
mod initial_server_stream_remote;
mod misc;
mod ping_frame;
mod process_receipt_frame_ty;
mod reset_stream_frame;
mod server_stream;
mod settings_frame;
mod stream_receiver;
mod stream_state;
#[cfg(test)]
mod tests;
#[cfg(feature = "web-socket")]
mod web_socket_over_stream;
mod window;
mod window_update_frame;
mod write_functions;

use crate::{
  collections::SingleTypeStorage,
  http::{DEFAULT_INITIAL_WINDOW_LEN, HttpRecvParams, MsgBufferString, Protocol, Request, U31},
  http2::settings_frame::SettingsFrame,
  misc::{Lease, LeaseMut, Usize},
  net::{ConnectionState, StreamReader, StreamWriter},
  sync::{Arc, AsyncMutex, AtomicU8},
  tls::{TlsMode, TlsStreamBridge, TlsStreamReader, TlsStreamWriter},
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
pub use http2_status::{Http2RecvStatus, Http2SendStatus};
pub use server_stream::ServerStream;
#[cfg(feature = "web-socket")]
pub use web_socket_over_stream::WebSocketOverStream;
pub use window::{Window, Windows};

const PREFACE: [u8; 24] = *b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

pub(crate) type Scorp = HashMap<U31, stream_receiver::StreamControlRecvParams>;
pub(crate) type Sovrp = HashMap<U31, stream_receiver::StreamOverallRecvParams>;

/// Negotiates initial "handshakes" or connections and also manages the creation of streams.
#[derive(Debug)]
pub struct Http2<SW, TM, const IS_CLIENT: bool> {
  inner: Arc<Http2Inner<SW, TM, IS_CLIENT>>,
}

impl<SW, TM, const IS_CLIENT: bool> Http2<SW, TM, IS_CLIENT>
where
  SW: StreamWriter,
  TM: TlsMode,
{
  /// See [`ConnectionState`].
  #[inline]
  pub fn connection_state(&self) -> ConnectionState {
    misc::connection_state(&self.inner.is_conn_open)
  }

  send_go_away_method!();

  #[cfg(all(feature = "http2-client-pool", feature = "tls"))]
  pub(crate) async fn swap_buffers(&mut self, hb: &mut Http2Buffer) {
    mem::swap(hb, self.inner.hd.lock().await.parts_mut().hb);
  }

  async fn manage_initial_params<SR, const HAS_PREFACE: bool>(
    mut hb: Http2Buffer,
    hrp: HttpRecvParams,
    stream_bridge: TlsStreamBridge<IS_CLIENT>,
    stream_reader: TlsStreamReader<SR, TM, IS_CLIENT>,
    mut stream_writer: TlsStreamWriter<SW, TM, IS_CLIENT>,
  ) -> crate::Result<(impl Future<Output = ()>, Self)>
  where
    SR: StreamReader,
  {
    let sf = SettingsFrame::from_hrp(hrp);
    let sf_buffer = &mut [0; 45];
    let sf_bytes = sf.bytes(sf_buffer);
    if hrp.initial_window_len() == DEFAULT_INITIAL_WINDOW_LEN {
      if HAS_PREFACE {
        misc::write_array([&PREFACE, sf_bytes], &mut stream_writer).await?;
      } else {
        misc::write_array([sf_bytes], &mut stream_writer).await?;
      }
    } else {
      let wuf = window_update_frame::WindowUpdateFrame::new(
        hrp.initial_window_len().wrapping_sub(DEFAULT_INITIAL_WINDOW_LEN).into(),
        U31::ZERO,
      )?;
      if HAS_PREFACE {
        let array = [&PREFACE, sf_bytes, &wuf.bytes()];
        misc::write_array(array, &mut stream_writer).await?;
      } else {
        misc::write_array([sf_bytes, &wuf.bytes()], &mut stream_writer).await?;
      }
    }
    hb.hpack_dec.set_max_bytes(hrp.max_hpack_len().0);
    hb.hpack_enc.set_max_dyn_super_bytes(hrp.max_hpack_len().1);
    let nrb = mem::take(&mut hb.nrb);
    let max_frame_len = hrp.max_frame_len();
    let inner = Arc::new(Http2Inner {
      hd: AsyncMutex::new(Http2Data::new(hb, hrp)),
      is_conn_open: stream_reader.connection_state_raw().clone(),
      wd: AsyncMutex::new(stream_writer),
    });
    Ok((
      frame_reader::frame_reader(inner.clone(), max_frame_len, nrb, stream_bridge, stream_reader),
      Self { inner },
    ))
  }
}

impl<SW, TM> Http2<SW, TM, false>
where
  SW: StreamWriter,
  TM: TlsMode,
{
  /// Accepts an initial connection sending the local parameters to the remote peer.
  #[inline]
  pub async fn accept<SR>(
    mut hb: Http2Buffer,
    hrp: HttpRecvParams,
    (stream_bridge, mut stream_reader, mut stream_writer): (
      TlsStreamBridge<false>,
      TlsStreamReader<SR, TM, false>,
      TlsStreamWriter<SW, TM, false>,
    ),
  ) -> crate::Result<(impl Future<Output = ()>, Self)>
  where
    SR: StreamReader,
  {
    hb.clear();
    let mut buffer = [0; PREFACE.len()];
    let _read = stream_reader.read(buffer.as_mut_slice().into()).await?;
    if buffer != PREFACE {
      let _rslt = stream_writer
        .write_all(
          &go_away_frame::GoAwayFrame::new(Http2ErrorCode::ProtocolError, U31::ZERO).bytes(),
        )
        .await;
      return Err(misc::protocol_err(Http2Error::NoPreface));
    }
    Self::manage_initial_params::<_, false>(hb, hrp, stream_bridge, stream_reader, stream_writer)
      .await
  }

  /// Awaits for an initial header to create a stream.
  ///
  /// Returns [`None`] if the network connection has been closed, either locally or externally.
  #[inline]
  pub async fn stream<T>(
    &self,
    mut cb: impl FnMut(Request<&mut MsgBufferString>, Option<Protocol>) -> T,
  ) -> crate::Result<Option<(ServerStream<SW, TM>, T)>> {
    let Self { inner } = self;
    let mut is_registered = false;
    let mut lock_pin = pin!(inner.hd.lock());
    poll_fn(|cx| {
      let mut guard = lock_pin!(cx, inner.hd, lock_pin);
      let hdpm = guard.parts_mut();
      let linger = hdpm.hp.linger();
      if misc::connection_state(&inner.is_conn_open).is_full_close() {
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
          linger,
          method,
          protocol,
          _trace_span!("New server stream", stream_id = %stream_id),
          stream_id,
        ),
        cb(Request::http2(method, &mut sorp.msg_buffer), protocol),
      ))))
    })
    .await
  }
}

impl<SW, TM> Http2<SW, TM, true>
where
  SW: StreamWriter,
  TM: TlsMode,
{
  /// Tries to connect to a server sending the local parameters.
  #[inline]
  pub async fn connect<SR>(
    mut hb: Http2Buffer,
    mut hrp: HttpRecvParams,
    (stream_bridge, stream_reader, stream_writer): (
      TlsStreamBridge<true>,
      TlsStreamReader<SR, TM, true>,
      TlsStreamWriter<SW, TM, true>,
    ),
  ) -> crate::Result<(impl Future<Output = ()>, Self)>
  where
    SR: StreamReader,
  {
    hb.clear();
    hrp = hrp.set_enable_connect_protocol(false);
    Self::manage_initial_params::<_, true>(hb, hrp, stream_bridge, stream_reader, stream_writer)
      .await
  }

  /// Opens a local stream.
  #[inline]
  pub async fn stream(&self) -> crate::Result<ClientStream<SW, TM>> {
    let Self { inner } = self;
    let mut hd_guard = inner.hd.lock().await;
    let hdpm = hd_guard.parts_mut();
    let linger = hdpm.hp.linger();
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
    *hdpm.last_stream_id = hdpm.last_stream_id.wrapping_add(U31::TWO);
    drop(hd_guard);
    Ok(ClientStream::new(inner.clone(), linger, span, stream_id))
  }
}

impl<SW, TM, const IS_CLIENT: bool> Lease<Http2<SW, TM, IS_CLIENT>> for Http2<SW, TM, IS_CLIENT> {
  #[inline]
  fn lease(&self) -> &Http2<SW, TM, IS_CLIENT> {
    self
  }
}

impl<SW, TM, const IS_CLIENT: bool> LeaseMut<Http2<SW, TM, IS_CLIENT>>
  for Http2<SW, TM, IS_CLIENT>
{
  #[inline]
  fn lease_mut(&mut self) -> &mut Http2<SW, TM, IS_CLIENT> {
    self
  }
}

impl<SW, TM, const IS_CLIENT: bool> SingleTypeStorage for Http2<SW, TM, IS_CLIENT> {
  type Item = (Http2Buffer, SW);
}

impl<SW, TM, const IS_CLIENT: bool> Clone for Http2<SW, TM, IS_CLIENT> {
  #[inline]
  fn clone(&self) -> Self {
    Self { inner: self.inner.clone() }
  }
}

#[derive(Debug)]
pub(crate) struct Http2Inner<SW, TM, const IS_CLIENT: bool> {
  pub(crate) hd: AsyncMutex<Http2Data<IS_CLIENT>>,
  pub(crate) is_conn_open: Arc<AtomicU8>,
  pub(crate) wd: AsyncMutex<TlsStreamWriter<SW, TM, IS_CLIENT>>,
}
