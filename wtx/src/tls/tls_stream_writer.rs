use crate::stream::StreamWriter;

pub struct TlsStreamWriter<SW> {
  pub(crate) stream_writer: SW,
}

impl<SW> StreamWriter for TlsStreamWriter<SW>
where
  SW: StreamWriter,
{
  #[inline]
  async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
    Ok(())
  }

  #[inline]
  async fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> crate::Result<()> {
    Ok(())
  }
}
