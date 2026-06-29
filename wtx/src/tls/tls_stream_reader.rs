use crate::{
  collections::{MaybeUninitSlice, ShortBoxSliceU16},
  misc::{ConnectionState, Either},
  stream::{BufStreamReader, StreamCommon, StreamReadItem, StreamReader},
  sync::{Arc, AtomicBool},
  tls::{
    TlsMode, TlsStreamBridge, TlsStreamBridgeData,
    key_schedule::KeyScheduleRead,
    misc::read_after_handshake_data,
    protocol::{alert::Alert, key_update::KeyUpdate, new_session_ticket::NewSessionTicket},
  },
};
use core::{num::NonZeroUsize, sync::atomic::Ordering};

/// Reader that can be used in concurrent scenarios.
#[derive(Debug)]
pub struct TlsStreamReader<SR, TM, const IS_CLIENT: bool> {
  pub(crate) connection_state: Arc<AtomicBool>,
  pub(crate) ksr: KeyScheduleRead,
  pub(crate) max_fragment_length: u16,
  pub(crate) new_session_ticket: Option<NewSessionTicket<ShortBoxSliceU16<u8>>>,
  pub(crate) plaintext_consumed: usize,
  pub(crate) plaintext_len: usize,
  pub(crate) reader_buffer: BufStreamReader,
  pub(crate) stream_bridge: TlsStreamBridge<IS_CLIENT>,
  pub(crate) stream_reader: SR,
  pub(crate) _tm: TM,
}

impl<SR, TM, const IS_CLIENT: bool> TlsStreamReader<SR, TM, IS_CLIENT> {
  /// See [`ConnectionState`].
  #[inline]
  pub fn connection_state(&self) -> ConnectionState {
    ConnectionState::from(self.connection_state.load(Ordering::Relaxed))
  }
}

impl<SR, TM, const IS_CLIENT: bool> StreamCommon for TlsStreamReader<SR, TM, IS_CLIENT> {}

impl<SR, TM, const IS_CLIENT: bool> StreamReader for TlsStreamReader<SR, TM, IS_CLIENT>
where
  SR: StreamReader,
  TM: TlsMode,
{
  #[inline]
  async fn read(
    &mut self,
    bytes: MaybeUninitSlice<'_, u8>,
  ) -> crate::Result<StreamReadItem<NonZeroUsize>> {
    read_after_handshake_data::<_, _, TM, IS_CLIENT>(
      (&self.stream_bridge, &self.connection_state),
      bytes,
      &mut self.ksr,
      self.max_fragment_length,
      &mut self.new_session_ticket,
      &mut self.plaintext_consumed,
      &mut self.plaintext_len,
      &mut self.reader_buffer,
      &mut self.stream_reader,
      alert_cb,
      key_update_cb,
    )
    .await
  }
}

impl<SR, TM, const IS_CLIENT: bool> Drop for TlsStreamReader<SR, TM, IS_CLIENT> {
  #[inline]
  fn drop(&mut self) {
    let _rslt = self.stream_bridge.data().update(|elem| (true, elem.1));
    self.stream_bridge.waker().wake();
  }
}

async fn alert_cb<SR, const IS_CLIENT: bool>(
  aux: &mut (&TlsStreamBridge<IS_CLIENT>, &Arc<AtomicBool>),
  alert: Alert,
  _: &mut SR,
) -> crate::Result<()> {
  if alert.description().is_warning() {
    let _rslt = aux
      .0
      .data()
      .update(|el| (el.0, Some(TlsStreamBridgeData { inner: Either::Left(alert.data_bytes()) })));
  }
  aux.1.store(true, Ordering::Relaxed);
  Ok(())
}

async fn key_update_cb<SR, const IS_CLIENT: bool>(
  aux: &mut (&TlsStreamBridge<IS_CLIENT>, &Arc<AtomicBool>),
  key_update: KeyUpdate,
  _: &mut SR,
) -> crate::Result<()> {
  let _rslt = aux.0.data().update(|el| {
    (el.0, Some(TlsStreamBridgeData { inner: Either::Right(key_update.data_bytes()) }))
  });
  Ok(())
}
