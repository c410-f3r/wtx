use crate::collections::Vector;

/// Dictates how compression should work.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CompressionFlush {
  #[default]
  /// Decides how much data to accumulate before producing output, in order to maximize
  /// compression.
  NoFlush = 0,
  /// All pending output is flushed to the output buffer and the output is aligned on a
  /// byte boundary.
  SyncFlush = 2,
}

/// Compression
pub trait Compression {
  /// If the implementation does nothing
  const IS_NOOP: bool = false;

  /// Compress
  ///
  /// Returns the amount of written bytes.
  fn compress(
    &mut self,
    flush: CompressionFlush,
    input: &[u8],
    output: &mut Vector<u8>,
  ) -> crate::Result<usize>;

  /// Returns the upper bound on the compressed size.
  fn compress_ub(&self, len: usize) -> usize;

  /// Prepare the instance for a new stream.
  fn reset(&mut self);
}

impl Compression for () {
  const IS_NOOP: bool = true;

  #[inline]
  fn compress(
    &mut self,
    _: CompressionFlush,
    _: &[u8],
    _: &mut Vector<u8>,
  ) -> crate::Result<usize> {
    Ok(0)
  }

  #[inline]
  fn compress_ub(&self, _: usize) -> usize {
    0
  }

  #[inline]
  fn reset(&mut self) {}
}

impl<T> Compression for Option<T>
where
  T: Compression,
{
  const IS_NOOP: bool = T::IS_NOOP;

  #[inline]
  fn compress(
    &mut self,
    flush: CompressionFlush,
    input: &[u8],
    output: &mut Vector<u8>,
  ) -> crate::Result<usize> {
    if let Some(elem) = self { elem.compress(flush, input, output) } else { Ok(0) }
  }

  #[inline]
  fn compress_ub(&self, len: usize) -> usize {
    if let Some(elem) = self { elem.compress_ub(len) } else { 0 }
  }

  #[inline]
  fn reset(&mut self) {
    if let Some(elem) = self {
      elem.reset();
    }
  }
}

#[cfg(feature = "zlib-rs")]
mod zlib_rs {
  use crate::{
    codec::{Compression, CompressionFlush},
    collections::Vector,
  };
  use zlib_rs::{Deflate, DeflateFlush, Status, compress_bound};

  impl Compression for Deflate {
    #[inline]
    fn compress(
      &mut self,
      flush: CompressionFlush,
      input: &[u8],
      output: &mut Vector<u8>,
    ) -> crate::Result<usize> {
      output.reserve((input.len() / 2).max(64))?;
      let mut curr_input = input;
      let mut total_written: u64 = 0;
      loop {
        let (_, uninit) = output.split_at_spare_mut();
        let uninit_len = uninit.len();
        let before_in = self.total_in();
        let before_out = self.total_out();
        let status = (*self).compress_uninit(curr_input, uninit, flush.into())?;
        let consumed = self.total_in().wrapping_sub(before_in).try_into()?;
        let written = self.total_out().wrapping_sub(before_out);
        let written_usize: usize = written.try_into()?;
        curr_input = curr_input.get(consumed..).unwrap_or_default();
        total_written = total_written.wrapping_add(written);
        let new_len = output.len().wrapping_add(written_usize);
        // SAFETY: `compress_uninit` just initialized `written` bytes
        unsafe {
          output.set_len(new_len);
        }
        match status {
          Status::BufError if curr_input.is_empty() => {
            return Ok(total_written.try_into()?);
          }
          Status::Ok if curr_input.is_empty() && written_usize < uninit_len => {
            return Ok(total_written.try_into()?);
          }
          Status::StreamEnd => {
            return Ok(total_written.try_into()?);
          }
          Status::BufError | Status::Ok => {}
        }
        output.reserve((output.capacity() / 2).max(64))?;
      }
    }

    #[inline]
    fn compress_ub(&self, len: usize) -> usize {
      compress_bound(len)
    }

    #[inline]
    fn reset(&mut self) {
      (*self).reset();
    }
  }

  impl From<CompressionFlush> for DeflateFlush {
    #[inline]
    fn from(value: CompressionFlush) -> Self {
      match value {
        CompressionFlush::NoFlush => DeflateFlush::NoFlush,
        CompressionFlush::SyncFlush => DeflateFlush::SyncFlush,
      }
    }
  }
}
