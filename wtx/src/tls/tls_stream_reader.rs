use crate::{
  collections::{MaybeUninitSlice, ShortBoxSliceU16},
  misc::{ConnectionState, Either},
  stream::{BufStreamReader, StreamCommon, StreamReader},
  tls::{
    TlsMode, TlsStreamBridge, TlsStreamBridgeData,
    key_schedule::KeyScheduleRead,
    misc::read_after_handshake_data,
    protocol::{alert::Alert, key_update::KeyUpdate, new_session_ticket::NewSessionTicket},
  },
};
use core::{hint::cold_path, num::NonZeroUsize};

/// Reader that can be used in concurrent scenarios.
#[derive(Debug)]
pub struct TlsStreamReader<SR, TM, const IS_CLIENT: bool> {
  pub(crate) connection_state: ConnectionState,
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
  ///
  /// The reader state is different from the writer state.
  #[inline]
  pub fn connection_state(&self) -> ConnectionState {
    self.connection_state
  }

  /// Sends a warning alert of type `CloseNotify`, closing the connection.
  #[inline]
  pub fn send_close_notify(&mut self) -> crate::Result<()> {
    self
      .stream_bridge
      .update(TlsStreamBridgeData::new(Either::Left(Alert::close_notify().data_bytes())));
    self.connection_state = ConnectionState::WriteClosed;
    Ok(())
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
    if TM::TY.is_plain_text() {
      return self.stream_reader.read(bytes).await;
    }
    if self.connection_state().cannot_read() {
      cold_path();
      return Ok(None);
    }
    read_after_handshake_data::<_, _, IS_CLIENT>(
      (&self.stream_bridge, &mut self.connection_state),
      bytes,
      &mut self.ksr,
      self.max_fragment_length,
      &mut self.new_session_ticket,
      &mut self.plaintext_consumed,
      &mut self.plaintext_len,
      &mut self.reader_buffer,
      &mut self.stream_reader,
      alert_cb,
      closed_conn_cb,
      key_update_cb,
    )
    .await
  }
}

async fn alert_cb<SR, const IS_CLIENT: bool>(
  aux: &mut (&TlsStreamBridge<IS_CLIENT>, &mut ConnectionState),
  alert: Alert,
  _: &mut SR,
) -> crate::Result<()> {
  *aux.1 = ConnectionState::ReadClosed;
  if alert.is_close_notify() {
    aux.0.update(TlsStreamBridgeData::new(Either::Left(alert.data_bytes())));
  }
  Ok(())
}

fn closed_conn_cb<const IS_CLIENT: bool>(
  aux: &mut (&TlsStreamBridge<IS_CLIENT>, &mut ConnectionState),
) {
  *aux.1 = ConnectionState::Closed;
}

async fn key_update_cb<SR, const IS_CLIENT: bool>(
  aux: &mut (&TlsStreamBridge<IS_CLIENT>, &mut ConnectionState),
  key_update: KeyUpdate,
  _: &mut SR,
) -> crate::Result<()> {
  aux.0.update(TlsStreamBridgeData::new(Either::Right(key_update.data_bytes())));
  Ok(())
}
