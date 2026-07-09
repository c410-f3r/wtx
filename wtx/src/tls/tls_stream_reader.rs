use crate::{
  collections::{MaybeUninitSlice, ShortBoxSliceU16},
  misc::{ConnectionState, Either},
  stream::{BufStreamReader, StreamCommon, StreamReader},
  sync::{Arc, AtomicU8, AtomicWaker},
  tls::{
    TlsMode, TlsStreamBridge, TlsStreamBridgeData,
    key_schedule::KeyScheduleRead,
    misc::read_after_handshake_data,
    protocol::{alert::Alert, key_update::KeyUpdate, new_session_ticket::NewSessionTicket},
  },
};
use core::{
  future::poll_fn, hint::cold_path, num::NonZeroUsize, pin::pin, sync::atomic::Ordering, task::Poll,
};

/// Reader that can be used in concurrent scenarios.
#[derive(Debug)]
pub struct TlsStreamReader<SR, TM, const IS_CLIENT: bool> {
  connection_state: Arc<AtomicU8>,
  ksr: KeyScheduleRead,
  max_fragment_length: u16,
  new_session_ticket: Option<NewSessionTicket<ShortBoxSliceU16<u8>>>,
  plaintext_consumed: usize,
  plaintext_len: usize,
  reader_buffer: BufStreamReader,
  reader_waker: Arc<AtomicWaker>,
  stream_bridge: TlsStreamBridge<IS_CLIENT>,
  stream_reader: SR,
  _tm: TM,
}

impl<SR, TM, const IS_CLIENT: bool> TlsStreamReader<SR, TM, IS_CLIENT> {
  #[inline]
  pub(crate) const fn new(
    connection_state: Arc<AtomicU8>,
    ksr: KeyScheduleRead,
    max_fragment_length: u16,
    new_session_ticket: Option<NewSessionTicket<ShortBoxSliceU16<u8>>>,
    plaintext_consumed: usize,
    plaintext_len: usize,
    reader_buffer: BufStreamReader,
    reader_waker: Arc<AtomicWaker>,
    stream_bridge: TlsStreamBridge<IS_CLIENT>,
    stream_reader: SR,
    _tm: TM,
  ) -> Self {
    Self {
      connection_state,
      ksr,
      max_fragment_length,
      new_session_ticket,
      plaintext_consumed,
      plaintext_len,
      reader_buffer,
      reader_waker,
      stream_bridge,
      stream_reader,
      _tm,
    }
  }

  /// See [`ConnectionState`].
  #[inline]
  pub fn connection_state(&self) -> ConnectionState {
    self.connection_state.load(Ordering::Relaxed).into()
  }

  /// Sends a warning alert of type `CloseNotify`, closing the connection.
  #[inline]
  pub fn send_close_notify(&mut self) -> crate::Result<()> {
    self
      .stream_bridge
      .update(TlsStreamBridgeData::new(Either::Left(Alert::close_notify().data_bytes())));
    self.connection_state.store(ConnectionState::WriteClosed.into(), Ordering::Relaxed);
    Ok(())
  }

  #[cfg(any(feature = "http2", feature = "web-socket"))]
  #[inline]
  pub(crate) const fn connection_state_raw(&self) -> &Arc<AtomicU8> {
    &self.connection_state
  }
}

impl<SR, TM, const IS_CLIENT: bool> StreamCommon for TlsStreamReader<SR, TM, IS_CLIENT> {}

impl<SR, TM, const IS_CLIENT: bool> StreamReader for TlsStreamReader<SR, TM, IS_CLIENT>
where
  SR: StreamReader,
  TM: TlsMode,
{
  #[inline]
  async fn read(&mut self, bytes: MaybeUninitSlice<'_, u8>) -> crate::Result<Option<NonZeroUsize>> {
    let Self {
      connection_state,
      ksr,
      max_fragment_length,
      new_session_ticket,
      plaintext_consumed,
      plaintext_len,
      reader_buffer,
      reader_waker,
      stream_bridge,
      stream_reader,
      _tm,
    } = self;
    let mut read_fut = pin!(async {
      if TM::TY.is_plain_text() {
        return stream_reader.read(bytes).await;
      }
      read_after_handshake_data::<_, _, IS_CLIENT>(
        (&*stream_bridge, &*connection_state),
        bytes,
        ksr,
        *max_fragment_length,
        new_session_ticket,
        plaintext_consumed,
        plaintext_len,
        reader_buffer,
        stream_reader,
        alert_cb,
        closed_conn_cb,
        key_update_cb,
      )
      .await
    });
    poll_fn(|cx| match read_fut.as_mut().poll(cx) {
      Poll::Ready(Ok(fi)) => Poll::Ready(Ok(fi)),
      Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
      Poll::Pending => {
        reader_waker.register(cx.waker());
        if ConnectionState::from(connection_state.load(Ordering::Relaxed)).cannot_read() {
          cold_path();
          return Poll::Ready(Ok(None));
        }
        Poll::Pending
      }
    })
    .await
  }
}

async fn alert_cb<SR, const IS_CLIENT: bool>(
  aux: &mut (&TlsStreamBridge<IS_CLIENT>, &Arc<AtomicU8>),
  alert: Alert,
  _: &mut SR,
) -> crate::Result<()> {
  aux.1.store(ConnectionState::ReadClosed.into(), Ordering::Relaxed);
  if alert.is_close_notify() {
    aux.0.update(TlsStreamBridgeData::new(Either::Left(alert.data_bytes())));
  }
  Ok(())
}

fn closed_conn_cb<const IS_CLIENT: bool>(aux: &mut (&TlsStreamBridge<IS_CLIENT>, &Arc<AtomicU8>)) {
  aux.1.store(ConnectionState::Closed.into(), Ordering::Relaxed);
}

async fn key_update_cb<SR, const IS_CLIENT: bool>(
  aux: &mut (&TlsStreamBridge<IS_CLIENT>, &Arc<AtomicU8>),
  key_update: KeyUpdate,
  _: &mut SR,
) -> crate::Result<()> {
  aux.0.update(TlsStreamBridgeData::new(Either::Right(key_update.data_bytes())));
  Ok(())
}
