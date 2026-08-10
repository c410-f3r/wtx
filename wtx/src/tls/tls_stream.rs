use crate::{
  _AFTER_CLOSE_TIMEOUT_MS,
  collections::{MaybeUninitSlice, ShortBoxSliceU16},
  futures::Sleep,
  net::{ConnectionState, Stream, StreamCommon, StreamReader, StreamWriter},
  sync::{Arc, AtomicBool, AtomicU8, AtomicWaker},
  tls::{
    AlertDescription, AlertLevel, TlsBuffer, TlsCtx, TlsError, TlsStreamBridge, TlsStreamReader,
    TlsStreamWriter,
    key_schedule::{KeySchedule, KeyScheduleWrite},
    misc::{
      manage_err_ad, manage_key_update, manage_user_canceled, read_after_handshake_data,
      write_payloads,
    },
    protocol::{
      alert::Alert,
      key_update::{KeyUpdate, KeyUpdateRequest},
      new_session_ticket::NewSessionTicket,
      record_content_ty::RecordContentTy,
    },
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
  task::{Poll, ready},
  time::Duration,
};

/// Transport Layer Security (TLS)
///
/// This structure assumes a previously established handshake.
#[derive(Debug)]
pub struct TlsStream<S, TCX, const IS_CLIENT: bool> {
  pub(crate) buffer: TlsBuffer,
  pub(crate) can_reply_key_update: bool,
  pub(crate) connection_state: ConnectionState,
  pub(crate) key_schedule: KeySchedule,
  pub(crate) key_updates: u8,
  pub(crate) max_fragment_length: u16,
  pub(crate) max_fragment_length_send: u16,
  pub(crate) new_session_ticket: Option<NewSessionTicket<ShortBoxSliceU16<u8>>>,
  pub(crate) phantom: PhantomData<TCX>,
  pub(crate) plaintext_consumed: usize,
  pub(crate) plaintext_len: usize,
  pub(crate) split_begin: usize,
  pub(crate) split_len: usize,
  pub(crate) stream: S,
  pub(crate) timer: Pin<Box<Sleep>>,
  pub(crate) warning_alerts: u8,
}

impl<S, TCX, const IS_CLIENT: bool> TlsStream<S, TCX, IS_CLIENT>
where
  S: Stream,
  TCX: TlsCtx,
{
  /// Creates a new instance with a stream that supposedly already performed a handshake.
  #[inline]
  pub fn new(
    buffer: TlsBuffer,
    key_schedule: KeySchedule,
    max_fragment_length: u16,
    max_fragment_length_send: u16,
    stream: S,
  ) -> crate::Result<Self> {
    Ok(Self {
      buffer,
      can_reply_key_update: true,
      connection_state: ConnectionState::Open,
      key_schedule,
      key_updates: 0,
      max_fragment_length,
      max_fragment_length_send,
      new_session_ticket: None,
      phantom: PhantomData,
      plaintext_consumed: 0,
      plaintext_len: 0,
      split_begin: 0,
      split_len: 0,
      stream,
      timer: Box::pin(Sleep::new(Duration::from_millis(_AFTER_CLOSE_TIMEOUT_MS))?),
      warning_alerts: 0,
    })
  }

  /// See [`ConnectionState`].
  #[inline]
  pub const fn connection_state(&self) -> ConnectionState {
    self.connection_state
  }

  /// Exports keying material
  #[inline]
  pub fn export_keying_material(
    &self,
    context: Option<&[u8]>,
    label: &[u8],
    output: &mut [u8],
  ) -> crate::Result<()> {
    KeySchedule::export_keying_material(
      self.key_schedule.cipher_suite(),
      context,
      self.key_schedule.exporter_secret(),
      label,
      output,
    )
  }

  /// Exports the read and write application traffic secrets.
  #[inline]
  pub fn export_traffic_secrets(&self) -> (&[u8], &[u8]) {
    (
      self.key_schedule.read().state().raw_traffic_secret(),
      self.key_schedule.write().state().raw_traffic_secret(),
    )
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
    self.stream.write_all(&key_update.record_bytes(kss)?).await?;
    kss.rotate()?;
    Ok(())
  }

  /// Sends a warning alert of type `CloseNotify`, closing the connection.
  #[inline]
  pub async fn send_close_notify(&mut self) -> crate::Result<()> {
    self
      .stream
      .write_all(&Alert::close_notify().record_bytes(self.key_schedule.write_mut().state_mut())?)
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

impl<S, TCX, const IS_CLIENT: bool> Stream for TlsStream<S, TCX, IS_CLIENT>
where
  S: Stream,
  TCX: TlsCtx,
{
  type BridgeOwned = TlsStreamBridge<IS_CLIENT>;
  type ReadHalfOwned = TlsStreamReader<S::ReadHalfOwned, TCX, IS_CLIENT>;
  type WriteHalfOwned = TlsStreamWriter<S::WriteHalfOwned, TCX, IS_CLIENT>;

  #[inline]
  fn into_split(
    self,
  ) -> crate::Result<(Self::BridgeOwned, Self::ReadHalfOwned, Self::WriteHalfOwned)> {
    let stream_bridge = TlsStreamBridge::new();
    let exporter_secret = *self.key_schedule.exporter_secret();
    let (ksr, ksw) = self.key_schedule.into_split();
    let (_, stream_reader, stream_writer) = self.stream.into_split()?;
    let common = Arc::new(TlsStreamCommon {
      can_reply_key_update: AtomicBool::new(self.can_reply_key_update),
      connection_state: AtomicU8::new(self.connection_state.into()),
      reader_waker: AtomicWaker::new(),
    });
    Ok((
      stream_bridge.clone(),
      TlsStreamReader::new(
        common.clone(),
        ksr,
        self.max_fragment_length,
        self.new_session_ticket,
        self.plaintext_consumed,
        self.plaintext_len,
        self.buffer.reader_buffer,
        stream_bridge,
        stream_reader,
      )?,
      TlsStreamWriter::new(
        common,
        exporter_secret,
        ksw,
        self.max_fragment_length_send,
        stream_writer,
        self.buffer.writer_buffer,
      ),
    ))
  }
}

impl<S, TCX, const IS_CLIENT: bool> StreamCommon for TlsStream<S, TCX, IS_CLIENT> {}

impl<S, TCX, const IS_CLIENT: bool> StreamReader for TlsStream<S, TCX, IS_CLIENT>
where
  S: Stream,
  TCX: TlsCtx,
{
  #[inline]
  async fn read(&mut self, bytes: MaybeUninitSlice<'_, u8>) -> crate::Result<Option<NonZeroUsize>> {
    let Self {
      buffer,
      can_reply_key_update,
      connection_state,
      key_schedule,
      key_updates,
      max_fragment_length_send: _,
      max_fragment_length,
      new_session_ticket,
      phantom: _,
      plaintext_consumed,
      plaintext_len,
      split_begin,
      split_len,
      stream,
      timer,
      warning_alerts,
    } = self;
    let local_connection_state = *connection_state;
    let mut read_fut = pin!(async {
      if TCX::TY.is_plain_text() {
        return stream.read(bytes).await;
      }
      if connection_state.cannot_read() {
        cold_path();
        return Ok(None);
      }
      let (ksr, ksw) = key_schedule.split_mut();
      let rslt = read_after_handshake_data::<_, _, IS_CLIENT>(
        Aux {
          connection_state: &mut *connection_state,
          can_reply_key_update,
          key_updates,
          ksw,
          warning_alerts,
        },
        bytes,
        ksr,
        *max_fragment_length,
        new_session_ticket,
        plaintext_consumed,
        plaintext_len,
        &mut buffer.reader_buffer,
        split_begin,
        split_len,
        stream,
        alert_cb,
        closed_conn_cb,
        key_update_cb,
        key_update_reset_cb,
      )
      .await;
      manage_err_ad(rslt, async |description| {
        let kss = key_schedule.write_mut().state_mut();
        stream.write_all(&Alert::fatal(description).record_bytes(kss)?[..]).await
      })
      .await
    });
    poll_fn(|cx| match read_fut.as_mut().poll(cx) {
      Poll::Ready(res) => Poll::Ready(res),
      Poll::Pending => {
        match local_connection_state {
          // Normal operation
          ConnectionState::Draining | ConnectionState::Open => Poll::Pending,
          // * An abrupt close signal was received/generated by us or the user abruptly closed
          // the connection.
          // * `ReadClosed` should be unreachable in sequential code.
          ConnectionState::ClosedAbruptly
          | ConnectionState::ClosedGracefully
          | ConnectionState::ReadClosed => {
            cold_path();
            Poll::Ready(Ok(None))
          }
          // Only called when the user sent a close notify and also decided to read data.
          ConnectionState::WriteClosed => {
            cold_path();
            let _rslt = ready!(timer.as_mut().poll(cx));
            Poll::Ready(Ok(None))
          }
        }
      }
    })
    .await
  }
}

impl<S, TCX, const IS_CLIENT: bool> StreamWriter for TlsStream<S, TCX, IS_CLIENT>
where
  S: StreamWriter,
  TCX: TlsCtx,
{
  #[inline]
  async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
    if TCX::TY.is_plain_text() {
      return self.stream.write_all(bytes).await;
    }
    if self.connection_state.cannot_write() {
      cold_path();
      return Ok(());
    }
    write_payloads(
      RecordContentTy::ApplicationData,
      self.key_schedule.write_mut(),
      self.max_fragment_length_send,
      &[bytes],
      &mut self.stream,
      &mut self.buffer.writer_buffer,
    )
    .await?;
    self.can_reply_key_update = true;
    Ok(())
  }

  #[inline]
  async fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> crate::Result<()> {
    if TCX::TY.is_plain_text() {
      return self.stream.write_all_vectored(bytes).await;
    }
    if self.connection_state.cannot_write() {
      cold_path();
      return Ok(());
    }
    write_payloads(
      RecordContentTy::ApplicationData,
      self.key_schedule.write_mut(),
      self.max_fragment_length_send,
      bytes,
      &mut self.stream,
      &mut self.buffer.writer_buffer,
    )
    .await?;
    self.can_reply_key_update = true;
    Ok(())
  }
}

async fn alert_cb<S>(aux: &mut Aux<'_>, alert: Alert, stream: &mut S) -> crate::Result<bool>
where
  S: Stream,
{
  match (alert.level(), alert.description()) {
    (AlertLevel::Warning, AlertDescription::CloseNotify) => {
      stream.write_all(&alert.record_bytes(aux.ksw.state_mut())?).await?;
      *aux.connection_state = ConnectionState::ClosedGracefully;
      Ok(true)
    }
    (AlertLevel::Warning, AlertDescription::UserCanceled) => {
      manage_user_canceled(aux.warning_alerts)
    }
    _ => Err(crate::Error::TlsErrorReply(TlsError::WrongAlert, AlertDescription::DecodeError)),
  }
}

// This branch is only entered when the peer closed the connection without an alert.
fn closed_conn_cb(aux: &mut Aux<'_>) {
  *aux.connection_state = ConnectionState::ClosedAbruptly;
}

async fn key_update_cb<S>(
  aux: &mut Aux<'_>,
  key_update: Option<KeyUpdate>,
  stream: &mut S,
) -> crate::Result<()>
where
  S: Stream,
{
  manage_key_update(aux.key_updates)?;
  if let Some(elem) = key_update
    && *aux.can_reply_key_update
  {
    let kss = aux.ksw.state_mut();
    stream.write_all(&elem.record_bytes(kss)?).await?;
    kss.rotate()?;
    *aux.can_reply_key_update = false;
  }
  Ok(())
}

async fn key_update_reset_cb<S>(aux: &mut Aux<'_>, _stream: &mut S) -> crate::Result<()>
where
  S: Stream,
{
  *aux.key_updates = 0;
  Ok(())
}

struct Aux<'any> {
  can_reply_key_update: &'any mut bool,
  connection_state: &'any mut ConnectionState,
  key_updates: &'any mut u8,
  ksw: &'any mut KeyScheduleWrite,
  warning_alerts: &'any mut u8,
}
