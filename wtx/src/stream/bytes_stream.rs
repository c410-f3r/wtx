use crate::{
  collections::{MaybeUninitSlice, Vector},
  stream::{Stream, StreamCommon, StreamReader, StreamWriter},
};
use core::{cmp::Ordering, num::NonZeroUsize};

/// Stores written data to transfer when read.
#[derive(Debug, Default)]
pub struct BytesStream {
  buffer: Vector<u8>,
  idx: usize,
}

impl BytesStream {
  /// Empties the internal buffer.
  #[inline]
  pub fn clear(&mut self) {
    self.buffer.clear();
    self.idx = 0;
  }
}

impl Stream for BytesStream {
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

impl StreamCommon for BytesStream {}

impl StreamReader for BytesStream {
  #[inline]
  async fn read(
    &mut self,
    mut bytes: MaybeUninitSlice<'_, u8>,
  ) -> crate::Result<Option<NonZeroUsize>> {
    let initialized = bytes.initialize_all_bytes();
    let working_buffer = self.buffer.get(self.idx..).unwrap_or_default();
    let working_buffer_len = working_buffer.len();
    Ok(NonZeroUsize::new(match working_buffer_len.cmp(&initialized.len()) {
      Ordering::Less => {
        initialized
          .get_mut(..working_buffer_len)
          .unwrap_or_default()
          .copy_from_slice(working_buffer);
        self.clear();
        working_buffer_len
      }
      Ordering::Equal => {
        initialized.copy_from_slice(working_buffer);
        self.clear();
        working_buffer_len
      }
      Ordering::Greater => {
        initialized.copy_from_slice(working_buffer.get(..initialized.len()).unwrap_or_default());
        self.idx = self.idx.wrapping_add(initialized.len());
        initialized.len()
      }
    }))
  }
}

impl StreamWriter for BytesStream {
  #[inline]
  async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
    self.buffer.extend_from_copyable_slice(bytes)?;
    Ok(())
  }

  #[inline]
  async fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> crate::Result<()> {
    for elem in bytes {
      self.buffer.extend_from_copyable_slice(elem)?;
    }
    Ok(())
  }
}
