use crate::{
  collection::{IndexedStorageMut as _, Vector},
  stream::{StreamReader, StreamWriter},
};
use core::cmp::Ordering;

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

impl StreamReader for BytesStream {
  #[inline]
  async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
    let working_buffer = self.buffer.get(self.idx..).unwrap_or_default();
    let working_buffer_len = working_buffer.len();
    Ok(match working_buffer_len.cmp(&bytes.len()) {
      Ordering::Less => {
        bytes.get_mut(..working_buffer_len).unwrap_or_default().copy_from_slice(working_buffer);
        self.clear();
        working_buffer_len
      }
      Ordering::Equal => {
        bytes.copy_from_slice(working_buffer);
        self.clear();
        working_buffer_len
      }
      Ordering::Greater => {
        bytes.copy_from_slice(working_buffer.get(..bytes.len()).unwrap_or_default());
        self.idx = self.idx.wrapping_add(bytes.len());
        bytes.len()
      }
    })
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
