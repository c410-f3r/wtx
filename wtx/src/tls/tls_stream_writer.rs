use crate::{
  collections::{ArrayVectorCopy, Vector},
  misc::Either,
  net::{ConnectionState, StreamCommon, StreamWriter},
  sync::Arc,
  tls::{
    KeySchedule, MAX_HASH_LEN, TlsCtx, TlsStreamBridgeData,
    key_schedule::KeyScheduleWrite,
    misc::write_payloads,
    protocol::{
      alert::Alert,
      key_update::{KeyUpdate, KeyUpdateRequest},
    },
    record_content_ty::RecordContentTy,
    tls_stream_common::TlsStreamCommon,
  },
};
use core::{hint::cold_path, marker::PhantomData, sync::atomic::Ordering};

/// Writer that can be used in concurrent scenarios.
#[derive(Debug)]
pub struct TlsStreamWriter<SW, TCX, const IS_CLIENT: bool> {
  common: Arc<TlsStreamCommon>,
  exporter_secret: ArrayVectorCopy<u8, MAX_HASH_LEN>,
  ksw: KeyScheduleWrite,
  max_fragment_length_send: u16,
  phantom: PhantomData<TCX>,
  stream_writer: SW,
  writer_buffer: Vector<u8>,
}

impl<SW, TCX, const IS_CLIENT: bool> TlsStreamWriter<SW, TCX, IS_CLIENT>
where
  SW: StreamWriter,
  TCX: TlsCtx,
{
  #[inline]
  pub(crate) const fn new(
    common: Arc<TlsStreamCommon>,
    exporter_secret: ArrayVectorCopy<u8, MAX_HASH_LEN>,
    ksw: KeyScheduleWrite,
    max_fragment_length_send: u16,
    stream_writer: SW,
    writer_buffer: Vector<u8>,
  ) -> Self {
    Self {
      common,
      exporter_secret,
      ksw,
      max_fragment_length_send,
      phantom: PhantomData,
      stream_writer,
      writer_buffer,
    }
  }

  /// Closes itself as well as the reader part without a graceful shutdown.
  #[inline]
  pub fn close_abruptly(&self) {
    self.common.connection_state.store(ConnectionState::ClosedAbruptly.into(), Ordering::Relaxed);
    self.common.reader_waker.wake();
  }

  /// See [`ConnectionState`].
  #[inline]
  pub fn connection_state(&self) -> ConnectionState {
    self.common.connection_state.load(Ordering::Relaxed).into()
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
      self.ksw.state().cipher_suite(),
      context,
      &self.exporter_secret,
      label,
      output,
    )
  }

  /// Exports the application traffic secrets.
  #[inline]
  pub fn export_traffic_secret(&self) -> &[u8] {
    self.ksw.state().raw_traffic_secret()
  }

  /// Writes the reply frame returned by `TlsStreamBridge::listen`. Returns `true` if the
  /// connection has been closed.
  #[inline]
  pub async fn manage_bridge_data(&mut self, data: TlsStreamBridgeData) -> crate::Result<bool> {
    let kss = self.ksw.state_mut();
    Ok(match data.frame() {
      Either::Left(elem) => {
        self.stream_writer.write_all(&Alert::record_bytes(elem, kss)?).await?;
        // The reader part received an alert, set itself as read closed, and signed us. This is
        // a graceful stop.
        self
          .common
          .connection_state
          .store(ConnectionState::ClosedGracefully.into(), Ordering::Relaxed);
        true
      }
      Either::Right(elem) => {
        self.stream_writer.write_all(&KeyUpdate::record_bytes(elem, kss)?).await?;
        kss.rotate()?;
        false
      }
    })
  }

  /// Refreshes the connection's keys through the sending of a `KeyUpdate` record.
  #[inline]
  pub async fn refresh_traffic_keys(&mut self) -> crate::Result<()> {
    let key_update = KeyUpdate::new(KeyUpdateRequest::UpdateRequested);
    let kss = self.ksw.state_mut();
    self.stream_writer.write_all(&key_update.record_bytes(kss)?).await?;
    kss.rotate()?;
    Ok(())
  }

  /// Sends a warning alert of type `CloseNotify`, gracefully closing the connection.
  #[inline]
  pub async fn send_close_notify(&mut self) -> crate::Result<()> {
    self
      .stream_writer
      .write_all(&Alert::close_notify().record_bytes(self.ksw.state_mut())?)
      .await?;
    self.common.connection_state.store(ConnectionState::WriteClosed.into(), Ordering::Relaxed);
    self.common.reader_waker.wake();
    Ok(())
  }

  /// References the inner stream responsible for sending and receiving data.
  #[inline]
  pub const fn stream(&self) -> &SW {
    &self.stream_writer
  }

  /// Mutable version of [`Self::stream`].
  #[inline]
  pub const fn stream_mut(&mut self) -> &mut SW {
    &mut self.stream_writer
  }

  #[cfg(feature = "web-socket")]
  #[inline]
  pub(crate) const fn common(&self) -> &Arc<TlsStreamCommon> {
    &self.common
  }
}

impl<SW, TCX, const IS_CLIENT: bool> StreamCommon for TlsStreamWriter<SW, TCX, IS_CLIENT> {}

impl<SW, TCX, const IS_CLIENT: bool> StreamWriter for TlsStreamWriter<SW, TCX, IS_CLIENT>
where
  SW: StreamWriter,
  TCX: TlsCtx,
{
  #[inline]
  async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
    if TCX::TY.is_plain_text() {
      return self.stream_writer.write_all(bytes).await;
    }
    if self.connection_state().cannot_write() {
      cold_path();
      return Ok(());
    }
    write_payloads(
      RecordContentTy::ApplicationData,
      &mut self.ksw,
      self.max_fragment_length_send,
      &[bytes],
      &mut self.stream_writer,
      &mut self.writer_buffer,
    )
    .await?;
    self.common.can_reply_key_update.store(true, Ordering::Relaxed);
    Ok(())
  }

  #[inline]
  async fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> crate::Result<()> {
    if TCX::TY.is_plain_text() {
      return self.stream_writer.write_all_vectored(bytes).await;
    }
    if self.connection_state().cannot_write() {
      cold_path();
      return Ok(());
    }
    write_payloads(
      RecordContentTy::ApplicationData,
      &mut self.ksw,
      self.max_fragment_length_send,
      bytes,
      &mut self.stream_writer,
      &mut self.writer_buffer,
    )
    .await?;
    self.common.can_reply_key_update.store(true, Ordering::Relaxed);
    Ok(())
  }
}
