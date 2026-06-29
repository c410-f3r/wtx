use core::num::NonZeroUsize;

use crate::{
  collections::MaybeUninitSlice,
  stream::{Stream, StreamCommon, StreamReadItem, StreamReader, StreamWriter},
};
use embassy_net::tcp::TcpSocket;

impl Stream for TcpSocket<'_> {
  type BridgeOwned = ();
  type ReadHalfOwned = ();
  type WriteHalfOwned = ();

  #[inline]
  fn into_split(
    self,
  ) -> crate::Result<(Self::BridgeOwned, Self::ReadHalfOwned, Self::WriteHalfOwned)> {
    Ok(((), (), ()))
  }
}

impl StreamCommon for TcpSocket<'_> {}

impl StreamReader for TcpSocket<'_> {
  #[inline]
  async fn read(
    &mut self,
    mut bytes: MaybeUninitSlice<'_, u8>,
  ) -> crate::Result<StreamReadItem<NonZeroUsize>> {
    Ok(StreamReadItem::from_opt(NonZeroUsize::new(
      (*self).read(bytes.initialize_all_bytes()).await?,
    )))
  }
}

impl StreamWriter for TcpSocket<'_> {
  #[inline]
  async fn write_all(&mut self, mut bytes: &[u8]) -> crate::Result<()> {
    _local_write_all!(bytes, Self::write(self, bytes).await);
    Ok(())
  }

  #[inline]
  async fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> crate::Result<()> {
    for elem in bytes {
      self.write_all(elem).await?;
    }
    Ok(())
  }
}
