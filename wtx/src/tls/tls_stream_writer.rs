use crate::{
  collections::Vector,
  misc::{ConnectionState, Either},
  stream::{StreamCommon, StreamWriter},
  sync::{Arc, AtomicU8, AtomicWaker},
  tls::{
    TlsMode, TlsStreamBridgeData,
    key_schedule::KeyScheduleWrite,
    misc::write_payloads,
    protocol::{alert::Alert, key_update::KeyUpdate, record_content_type::RecordContentType},
  },
};
use core::{hint::cold_path, sync::atomic::Ordering};

/// Writer that can be used in concurrent scenarios.
#[derive(Debug)]
pub struct TlsStreamWriter<SW, TM, const IS_CLIENT: bool> {
  connection_state: Arc<AtomicU8>,
  ksw: KeyScheduleWrite,
  max_fragment_length: u16,
  reader_waker: Arc<AtomicWaker>,
  stream_writer: SW,
  _tm: TM,
  writer_buffer: Vector<u8>,
}

impl<SW, TM, const IS_CLIENT: bool> TlsStreamWriter<SW, TM, IS_CLIENT>
where
  SW: StreamWriter,
  TM: TlsMode,
{
  #[inline]
  pub(crate) const fn new(
    connection_state: Arc<AtomicU8>,
    ksw: KeyScheduleWrite,
    max_fragment_length: u16,
    reader_waker: Arc<AtomicWaker>,
    stream_writer: SW,
    _tm: TM,
    writer_buffer: Vector<u8>,
  ) -> Self {
    Self {
      connection_state,
      ksw,
      max_fragment_length,
      reader_waker,
      stream_writer,
      _tm,
      writer_buffer,
    }
  }

  /// Closes itself as well as the reader part
  #[inline]
  pub fn close(&self) {
    self.connection_state.store(ConnectionState::Closed.into(), Ordering::Relaxed);
    self.reader_waker.wake();
  }

  /// See [`ConnectionState`].
  #[inline]
  pub fn connection_state(&self) -> ConnectionState {
    self.connection_state.load(Ordering::Relaxed).into()
  }

  /// Writes the reply frame returned by `TlsStreamBridge::listen`. Returns `true` if the
  /// connection has been closed.
  #[inline]
  pub async fn manage_bridge_data(&mut self, data: TlsStreamBridgeData) -> crate::Result<bool> {
    let kss = self.ksw.state_mut();
    Ok(match data.frame() {
      Either::Left(elem) => {
        self.stream_writer.write_all(&Alert::record_bytes(elem, kss)?).await?;
        self.close();
        true
      }
      Either::Right(elem) => {
        self.stream_writer.write_all(&KeyUpdate::record_bytes(elem, kss)?).await?;
        kss.rotate()?;
        false
      }
    })
  }

  #[cfg(feature = "web-socket")]
  #[inline]
  pub(crate) const fn connection_state_raw(&self) -> &Arc<AtomicU8> {
    &self.connection_state
  }
}

impl<SW, TM, const IS_CLIENT: bool> StreamCommon for TlsStreamWriter<SW, TM, IS_CLIENT> {}

impl<SW, TM, const IS_CLIENT: bool> StreamWriter for TlsStreamWriter<SW, TM, IS_CLIENT>
where
  SW: StreamWriter,
  TM: TlsMode,
{
  #[inline]
  async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
    if TM::TY.is_plain_text() {
      return self.stream_writer.write_all(bytes).await;
    }
    if self.connection_state().cannot_write() {
      cold_path();
      return Ok(());
    }
    write_payloads(
      RecordContentType::ApplicationData,
      &mut self.ksw,
      self.max_fragment_length,
      &[bytes],
      &mut self.stream_writer,
      &mut self.writer_buffer,
    )
    .await
  }

  #[inline]
  async fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> crate::Result<()> {
    if TM::TY.is_plain_text() {
      return self.stream_writer.write_all_vectored(bytes).await;
    }
    if self.connection_state().cannot_write() {
      cold_path();
      return Ok(());
    }
    write_payloads(
      RecordContentType::ApplicationData,
      &mut self.ksw,
      self.max_fragment_length,
      bytes,
      &mut self.stream_writer,
      &mut self.writer_buffer,
    )
    .await
  }
}
