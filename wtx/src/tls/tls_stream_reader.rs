use crate::{
  _AFTER_CLOSE_TIMEOUT_MS,
  collections::{MaybeUninitSlice, ShortBoxSliceU16},
  futures::Sleep,
  misc::Either,
  net::{BufStreamReader, ConnectionState, StreamCommon, StreamReader},
  sync::Arc,
  tls::{
    AlertDescription, AlertLevel, TlsCtx, TlsError, TlsStreamBridge, TlsStreamBridgeData,
    key_schedule::KeyScheduleRead,
    misc::{manage_err_ad, manage_key_update, manage_user_canceled, read_after_handshake_data},
    protocol::{alert::Alert, key_update::KeyUpdate, new_session_ticket::NewSessionTicket},
    tls_stream_common::TlsStreamCommon,
  },
};
use alloc::boxed::Box;
use core::{
  future::poll_fn,
  hint::cold_path,
  marker::PhantomData,
  num::NonZeroUsize,
  pin::{Pin, pin},
  sync::atomic::Ordering,
  task::{Poll, ready},
  time::Duration,
};

/// Reader that can be used in concurrent scenarios.
#[derive(Debug)]
pub struct TlsStreamReader<SR, TCX, const IS_CLIENT: bool> {
  common: Arc<TlsStreamCommon>,
  key_updates: u8,
  ksr: KeyScheduleRead,
  max_fragment_length: u16,
  new_session_ticket: Option<NewSessionTicket<ShortBoxSliceU16<u8>>>,
  phantom: PhantomData<TCX>,
  plaintext_consumed: usize,
  plaintext_len: usize,
  reader_buffer: BufStreamReader,
  split_begin: usize,
  split_len: usize,
  stream_bridge: TlsStreamBridge<IS_CLIENT>,
  stream_reader: SR,
  timer: Pin<Box<Sleep>>,
  warning_alerts: u8,
}

impl<SR, TCX, const IS_CLIENT: bool> TlsStreamReader<SR, TCX, IS_CLIENT> {
  #[inline]
  pub(crate) fn new(
    common: Arc<TlsStreamCommon>,
    ksr: KeyScheduleRead,
    max_fragment_length: u16,
    new_session_ticket: Option<NewSessionTicket<ShortBoxSliceU16<u8>>>,
    plaintext_consumed: usize,
    plaintext_len: usize,
    reader_buffer: BufStreamReader,
    stream_bridge: TlsStreamBridge<IS_CLIENT>,
    stream_reader: SR,
  ) -> crate::Result<Self> {
    Ok(Self {
      common,
      key_updates: 0,
      ksr,
      max_fragment_length,
      new_session_ticket,
      phantom: PhantomData,
      plaintext_consumed,
      plaintext_len,
      reader_buffer,
      split_begin: 0,
      split_len: 0,
      stream_bridge,
      stream_reader,
      timer: Box::pin(Sleep::new(Duration::from_millis(_AFTER_CLOSE_TIMEOUT_MS))?),
      warning_alerts: 0,
    })
  }

  /// Closes itself as well as the write part without a graceful shutdown.
  //
  // There is nothing to write to the peer so we don't wake the writer side.
  #[inline]
  pub fn close_abruptly(&self) {
    self.common.connection_state.store(ConnectionState::ClosedAbruptly.into(), Ordering::Relaxed);
  }

  /// See [`ConnectionState`].
  #[inline]
  pub fn connection_state(&self) -> ConnectionState {
    self.common.connection_state.load(Ordering::Relaxed).into()
  }

  /// Exports the application traffic secrets.
  #[inline]
  pub fn export_traffic_secret(&self) -> &[u8] {
    self.ksr.state().raw_traffic_secret()
  }

  /// Sends a warning alert of type `CloseNotify`, gracefully closing the connection.
  #[inline]
  pub fn send_close_notify(&self) -> crate::Result<()> {
    self.common.connection_state.store(ConnectionState::WriteClosed.into(), Ordering::Relaxed);
    self.stream_bridge.update(TlsStreamBridgeData::new(Either::Left(Alert::close_notify())));
    Ok(())
  }

  #[cfg(any(feature = "http2", feature = "web-socket"))]
  #[inline]
  pub(crate) const fn common(&self) -> &Arc<TlsStreamCommon> {
    &self.common
  }
}

impl<SR, TCX, const IS_CLIENT: bool> StreamCommon for TlsStreamReader<SR, TCX, IS_CLIENT> {}

impl<SR, TCX, const IS_CLIENT: bool> StreamReader for TlsStreamReader<SR, TCX, IS_CLIENT>
where
  SR: StreamReader,
  TCX: TlsCtx,
{
  #[inline]
  async fn read(&mut self, bytes: MaybeUninitSlice<'_, u8>) -> crate::Result<Option<NonZeroUsize>> {
    let Self {
      common,
      key_updates,
      ksr,
      max_fragment_length,
      new_session_ticket,
      phantom: _,
      plaintext_consumed,
      plaintext_len,
      reader_buffer,
      split_begin,
      split_len,
      stream_bridge,
      stream_reader,
      timer,
      warning_alerts,
    } = self;
    let mut read_fut = pin!(async {
      if TCX::TY.is_plain_text() {
        return stream_reader.read(bytes).await;
      }
      let rslt = read_after_handshake_data::<_, _, IS_CLIENT>(
        Aux { common, key_updates, stream_bridge, warning_alerts },
        bytes,
        ksr,
        *max_fragment_length,
        new_session_ticket,
        plaintext_consumed,
        plaintext_len,
        reader_buffer,
        split_begin,
        split_len,
        stream_reader,
        alert_cb,
        closed_conn_cb,
        key_update_cb,
        key_update_reset_cb,
      )
      .await;
      manage_err_ad(rslt, async |description| {
        stream_bridge.update(TlsStreamBridgeData::new(Either::Left(Alert::new(
          AlertLevel::Fatal,
          description,
        ))));
        common.connection_state.store(ConnectionState::ClosedAbruptly.into(), Ordering::Relaxed);
        Ok(())
      })
      .await
    });
    poll_fn(|cx| match read_fut.as_mut().poll(cx) {
      Poll::Ready(res) => Poll::Ready(res),
      Poll::Pending => {
        common.reader_waker.register(cx.waker());
        let current_state = common.connection_state.load(Ordering::Relaxed);
        match ConnectionState::from(current_state) {
          // Normal operation
          ConnectionState::Draining | ConnectionState::Open => Poll::Pending,
          // * An abrupt close signal was received/generated by us or the writer side abruptly
          // closed the connection.
          // * A graceful close signal was received (We CANNOT read close ourselves). The writer
          // side should have received a closing signal
          ConnectionState::ClosedAbruptly
          | ConnectionState::ClosedGracefully
          | ConnectionState::ReadClosed => {
            cold_path();
            Poll::Ready(Ok(None))
          }
          // After user interaction the writer side set itself as write closed then woke us. This
          // is a graceful stop.
          ConnectionState::WriteClosed => {
            cold_path();
            let _rslt = ready!(timer.as_mut().poll(cx));
            common
              .connection_state
              .store(ConnectionState::ClosedGracefully.into(), Ordering::Relaxed);
            Poll::Ready(Ok(None))
          }
        }
      }
    })
    .await
  }
}

async fn alert_cb<SR, const IS_CLIENT: bool>(
  aux: &mut Aux<'_, IS_CLIENT>,
  alert: Alert,
  _: &mut SR,
) -> crate::Result<bool> {
  match (alert.level(), alert.description()) {
    (AlertLevel::Warning, AlertDescription::CloseNotify) => {
      aux.common.connection_state.store(ConnectionState::ReadClosed.into(), Ordering::Relaxed);
      aux.stream_bridge.update(TlsStreamBridgeData::new(Either::Left(alert)));
      Ok(true)
    }
    (AlertLevel::Warning, AlertDescription::UserCanceled) => {
      manage_user_canceled(aux.warning_alerts)
    }
    _ => Err(crate::Error::TlsErrorReply(TlsError::WrongAlert, AlertDescription::DecodeError)),
  }
}

// This branch is only entered when the peer closed the connection without an alert.
fn closed_conn_cb<const IS_CLIENT: bool>(aux: &mut Aux<'_, IS_CLIENT>) {
  aux.common.connection_state.store(ConnectionState::ClosedAbruptly.into(), Ordering::Relaxed);
}

async fn key_update_cb<SR, const IS_CLIENT: bool>(
  aux: &mut Aux<'_, IS_CLIENT>,
  key_update: Option<KeyUpdate>,
  _: &mut SR,
) -> crate::Result<()> {
  manage_key_update(aux.key_updates)?;
  if let Some(elem) = key_update
    && aux.common.can_reply_key_update.load(Ordering::Relaxed)
  {
    aux.stream_bridge.update(TlsStreamBridgeData::new(Either::Right(elem)));
    aux.common.can_reply_key_update.store(false, Ordering::Relaxed);
  }
  Ok(())
}

async fn key_update_reset_cb<SR, const IS_CLIENT: bool>(
  aux: &mut Aux<'_, IS_CLIENT>,
  _: &mut SR,
) -> crate::Result<()> {
  *aux.key_updates = 0;
  Ok(())
}

struct Aux<'any, const IS_CLIENT: bool> {
  common: &'any Arc<TlsStreamCommon>,
  key_updates: &'any mut u8,
  stream_bridge: &'any TlsStreamBridge<IS_CLIENT>,
  warning_alerts: &'any mut u8,
}
