use crate::stream::{StreamCommon, StreamReader};

pub struct TlsStreamReader<SR> {
  pub(crate) stream_reader: SR,
}

impl<SR> StreamCommon for TlsStreamReader<SR> {}

impl<SR> StreamReader for TlsStreamReader<SR>
where
  SR: StreamReader,
{
  #[inline]
  async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
    Ok(0)
  }
}
