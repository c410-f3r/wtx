use crate::{
  collections::{MaybeUninitSlice, ShortBoxSliceU16},
  misc::ConnectionState,
  stream::{Stream, StreamCommon, StreamReader, StreamWriter},
  sync::{Arc, AtomicU8, AtomicWaker},
  tls::{
    TlsBuffer, TlsMode, TlsStreamBridge, TlsStreamReader, TlsStreamWriter,
    key_schedule::{KeySchedule, KeyScheduleWrite},
    misc::{read_after_handshake_data, write_payloads},
    protocol::{
      alert::Alert,
      key_update::{KeyUpdate, KeyUpdateRequest},
      new_session_ticket::NewSessionTicket,
      record_content_type::RecordContentType,
    },
  },
};
use core::{hint::cold_path, num::NonZeroUsize};

/// Transport Layer Security (TLS)
///
/// This structure assumes a previously established handshake.
#[derive(Debug)]
pub struct TlsStream<S, TM, const IS_CLIENT: bool> {
  pub(crate) buffer: TlsBuffer,
  pub(crate) connection_state: ConnectionState,
  pub(crate) key_schedule: KeySchedule,
  pub(crate) max_fragment_length: u16,
  pub(crate) new_session_ticket: Option<NewSessionTicket<ShortBoxSliceU16<u8>>>,
  pub(crate) plaintext_consumed: usize,
  pub(crate) plaintext_len: usize,
  pub(crate) stream: S,
  pub(crate) _tm: TM,
}

impl<S, TM, const IS_CLIENT: bool> TlsStream<S, TM, IS_CLIENT>
where
  S: Stream,
  TM: TlsMode,
{
  /// Creates a new instance with a stream that supposedly already performed a handshake.
  #[inline]
  pub fn new(
    buffer: TlsBuffer,
    key_schedule: KeySchedule,
    max_fragment_length: u16,
    stream: S,
    tm: TM,
  ) -> Self {
    Self {
      buffer,
      connection_state: ConnectionState::Open,
      key_schedule,
      max_fragment_length,
      new_session_ticket: None,
      plaintext_consumed: 0,
      plaintext_len: 0,
      stream,
      _tm: tm,
    }
  }

  /// See [`ConnectionState`].
  #[inline]
  pub const fn connection_state(&self) -> ConnectionState {
    self.connection_state
  }

  /// Returns the last received [`NewSessionTicket`], if any.
  ///
  /// NO-OP if `IS_CLIENT` is `false`.
  #[inline]
  pub const fn new_session_ticket(&self) -> &Option<NewSessionTicket<ShortBoxSliceU16<u8>>> {
    &self.new_session_ticket
  }

  /// Refreshes the connection's keys through the sending of a `KeyUpdate` record.
  #[inline]
  pub async fn refresh_traffic_keys(&mut self) -> crate::Result<()> {
    let key_update = KeyUpdate::new(KeyUpdateRequest::UpdateRequested);
    let kss = self.key_schedule.write_mut().state_mut();
    self.stream.write_all(&KeyUpdate::record_bytes(key_update.data_bytes(), kss)?).await?;
    kss.rotate()?;
    Ok(())
  }

  /// Sends a warning alert of type `CloseNotify`, closing the connection.
  #[inline]
  pub async fn send_close_notify(&mut self) -> crate::Result<()> {
    self
      .stream
      .write_all(&Alert::record_bytes(
        Alert::close_notify().data_bytes(),
        self.key_schedule.write_mut().state_mut(),
      )?)
      .await?;
    self.connection_state = ConnectionState::WriteClosed;
    Ok(())
  }

  /// References the inner stream responsible for sending and receiving data.
  #[inline]
  pub const fn stream(&self) -> &S {
    &self.stream
  }

  /// Mutable version of [`Self::stream`].
  #[inline]
  pub const fn stream_mut(&mut self) -> &mut S {
    &mut self.stream
  }
}

impl<S, TM, const IS_CLIENT: bool> Stream for TlsStream<S, TM, IS_CLIENT>
where
  S: Stream,
  TM: TlsMode,
{
  type BridgeOwned = TlsStreamBridge<IS_CLIENT>;
  type ReadHalfOwned = TlsStreamReader<S::ReadHalfOwned, TM, IS_CLIENT>;
  type WriteHalfOwned = TlsStreamWriter<S::WriteHalfOwned, TM, IS_CLIENT>;

  #[inline]
  fn into_split(
    self,
  ) -> crate::Result<(Self::BridgeOwned, Self::ReadHalfOwned, Self::WriteHalfOwned)> {
    let stream_bridge = TlsStreamBridge::new();
    let (ksr, ksw) = self.key_schedule.into_split();
    let (_, stream_reader, stream_writer) = self.stream.into_split()?;
    let connection_state = Arc::new(AtomicU8::new(self.connection_state.into()));
    let reader_waker = Arc::new(AtomicWaker::new());
    Ok((
      stream_bridge.clone(),
      TlsStreamReader::new(
        connection_state.clone(),
        ksr,
        self.max_fragment_length,
        self.new_session_ticket,
        self.plaintext_consumed,
        self.plaintext_len,
        self.buffer.reader_buffer,
        reader_waker.clone(),
        stream_bridge,
        stream_reader,
        self._tm.clone(),
      ),
      TlsStreamWriter::new(
        connection_state,
        ksw,
        self.max_fragment_length,
        reader_waker,
        stream_writer,
        self._tm,
        self.buffer.writer_buffer,
      ),
    ))
  }
}

impl<S, TM, const IS_CLIENT: bool> StreamCommon for TlsStream<S, TM, IS_CLIENT> {}

impl<S, TM, const IS_CLIENT: bool> StreamReader for TlsStream<S, TM, IS_CLIENT>
where
  S: Stream,
  TM: TlsMode,
{
  #[inline]
  async fn read(&mut self, bytes: MaybeUninitSlice<'_, u8>) -> crate::Result<Option<NonZeroUsize>> {
    if TM::TY.is_plain_text() {
      return self.stream.read(bytes).await;
    }
    if self.connection_state.cannot_read() {
      cold_path();
      return Ok(None);
    }
    let (ksr, ksw) = self.key_schedule.split_mut();
    read_after_handshake_data::<_, _, IS_CLIENT>(
      (&mut self.connection_state, ksw),
      bytes,
      ksr,
      self.max_fragment_length,
      &mut self.new_session_ticket,
      &mut self.plaintext_consumed,
      &mut self.plaintext_len,
      &mut self.buffer.reader_buffer,
      &mut self.stream,
      alert_cb,
      closed_conn_cb,
      key_update_cb,
    )
    .await
  }
}

impl<S, TM, const IS_CLIENT: bool> StreamWriter for TlsStream<S, TM, IS_CLIENT>
where
  S: StreamWriter,
  TM: TlsMode,
{
  #[inline]
  async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
    if TM::TY.is_plain_text() {
      return self.stream.write_all(bytes).await;
    }
    if self.connection_state.cannot_write() {
      cold_path();
      return Ok(());
    }
    write_payloads(
      RecordContentType::ApplicationData,
      self.key_schedule.write_mut(),
      self.max_fragment_length,
      &[bytes],
      &mut self.stream,
      &mut self.buffer.writer_buffer,
    )
    .await
  }

  #[inline]
  async fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> crate::Result<()> {
    if TM::TY.is_plain_text() {
      return self.stream.write_all_vectored(bytes).await;
    }
    if self.connection_state.cannot_write() {
      cold_path();
      return Ok(());
    }
    write_payloads(
      RecordContentType::ApplicationData,
      self.key_schedule.write_mut(),
      self.max_fragment_length,
      bytes,
      &mut self.stream,
      &mut self.buffer.writer_buffer,
    )
    .await
  }
}

async fn alert_cb<S>(
  aux: &mut (&mut ConnectionState, &mut KeyScheduleWrite),
  alert: Alert,
  stream: &mut S,
) -> crate::Result<()>
where
  S: Stream,
{
  if alert.is_close_notify() {
    stream.write_all(&Alert::record_bytes(alert.data_bytes(), aux.1.state_mut())?).await?;
  }
  *aux.0 = ConnectionState::Closed;
  Ok(())
}

fn closed_conn_cb(aux: &mut (&mut ConnectionState, &mut KeyScheduleWrite)) {
  *aux.0 = ConnectionState::Closed;
}

async fn key_update_cb<S>(
  aux: &mut (&mut ConnectionState, &mut KeyScheduleWrite),
  key_update: KeyUpdate,
  stream: &mut S,
) -> crate::Result<()>
where
  S: Stream,
{
  let kss = aux.1.state_mut();
  stream.write_all(&KeyUpdate::record_bytes(key_update.data_bytes(), kss)?).await?;
  kss.rotate()?;
  Ok(())
}
