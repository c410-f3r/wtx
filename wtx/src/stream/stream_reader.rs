use core::{future, num::NonZeroUsize};

use crate::{
  collections::MaybeUninitSlice,
  stream::{StreamCommon, StreamReadItem},
};

/// A stream of values sent asynchronously.
pub trait StreamReader: StreamCommon {
  /// Pulls some bytes from this source into the specified buffer, returning how many bytes
  /// were read.
  fn read(
    &mut self,
    bytes: MaybeUninitSlice<'_, u8>,
  ) -> impl Future<Output = crate::Result<StreamReadItem<NonZeroUsize>>>;

  /// Reads and at the same time discards exactly `len` bytes.
  #[inline]
  fn read_skip(&mut self, len: usize) -> impl Future<Output = crate::Result<()>> {
    async move {
      let mut buffer = [0; 32];
      let mut counter = len;
      for _ in 0..len {
        if counter == 0 {
          break;
        }
        let slice = if let Some(el) = buffer.get_mut(..counter) { el } else { &mut buffer[..] };
        let Some(read) = self.read(slice.into()).await?.opt() else {
          return Err(crate::Error::UnexpectedStreamReadEOF);
        };
        counter = counter.wrapping_sub(read.get());
      }
      Ok(())
    }
  }
}

impl<T> StreamReader for &mut T
where
  T: StreamReader,
{
  #[inline]
  async fn read(
    &mut self,
    bytes: MaybeUninitSlice<'_, u8>,
  ) -> crate::Result<StreamReadItem<NonZeroUsize>> {
    (**self).read(bytes).await
  }
}

impl StreamReader for () {
  #[inline]
  fn read(
    &mut self,
    _: MaybeUninitSlice<'_, u8>,
  ) -> impl Future<Output = crate::Result<StreamReadItem<NonZeroUsize>>> {
    future::ready(Ok(StreamReadItem::empty_cold()))
  }
}
