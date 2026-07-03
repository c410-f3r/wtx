use crate::{
  collections::Vector,
  misc::{ConnectionState, Either},
  stream::{StreamCommon, StreamWriter},
  sync::{Arc, AtomicBool},
  tls::{
    TlsMode, TlsStreamBridgeData,
    key_schedule::KeyScheduleWrite,
    misc::write_data,
    protocol::{alert::Alert, key_update::KeyUpdate, record_content_type::RecordContentType},
  },
};
use core::sync::atomic::Ordering;

/// Writer that can be used in concurrent scenarios.
#[derive(Debug)]
pub struct TlsStreamWriter<SW, TM, const IS_CLIENT: bool> {
  pub(crate) connection_state: Arc<AtomicBool>,
  pub(crate) ksw: KeyScheduleWrite,
  pub(crate) max_fragment_length: u16,
  pub(crate) stream_writer: SW,
  pub(crate) _tm: TM,
  pub(crate) writer_buffer: Vector<u8>,
}

impl<SW, TM, const IS_CLIENT: bool> TlsStreamWriter<SW, TM, IS_CLIENT> {
  /// See [`ConnectionState`].
  #[inline]
  pub fn connection_state(&self) -> ConnectionState {
    ConnectionState::from(self.connection_state.load(Ordering::Relaxed))
  }
}

impl<SW, TM, const IS_CLIENT: bool> TlsStreamWriter<SW, TM, IS_CLIENT>
where
  SW: StreamWriter,
  TM: TlsMode,
{
  /// See [`ConnectionState`].
  #[inline]
  pub async fn manage_bridge_data(&mut self, data: TlsStreamBridgeData) -> crate::Result<()> {
    let kss = self.ksw.state_mut();
    match data.inner {
      Either::Left(elem) => {
        self.stream_writer.write_all(&Alert::record_bytes(elem, kss)?).await?;
      }
      Either::Right(elem) => {
        self.stream_writer.write_all(&KeyUpdate::record_bytes(elem, kss)?).await?;
        kss.rotate()?;
      }
    }
    kss.increment_counter();
    Ok(())
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
    write_data(
      &[bytes],
      RecordContentType::ApplicationData,
      &mut self.ksw,
      self.max_fragment_length,
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
    write_data(
      bytes,
      RecordContentType::ApplicationData,
      &mut self.ksw,
      self.max_fragment_length,
      &mut self.stream_writer,
      &mut self.writer_buffer,
    )
    .await
  }
}
