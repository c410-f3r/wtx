use crate::collections::{LinearStorageLen, Vector};

/// Dictates how decompression should work.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DecompressionFlush {
  #[default]
  /// Decides how much data to accumulate before producing output, in order to maximize
  /// decompression.
  NoFlush = 0,
  /// All pending output is flushed to the output buffer and the output is aligned on a
  /// byte boundary.
  SyncFlush = 2,
}

/// Decompression
pub trait Decompression {
  /// If the implementation does nothing
  const IS_NOOP: bool = false;

  /// Decompress
  ///
  /// Returns the number of written bytes.
  fn decompress<L>(
    &mut self,
    flush: DecompressionFlush,
    input: &[u8],
    output: &mut Vector<L, u8>,
    upper_bound: L,
  ) -> crate::Result<L>
  where
    L: LinearStorageLen;

  /// Prepare the instance for a new stream.
  fn reset(&mut self);
}

impl Decompression for () {
  const IS_NOOP: bool = true;

  #[inline]
  fn decompress<L>(
    &mut self,
    _: DecompressionFlush,
    _: &[u8],
    _: &mut Vector<L, u8>,
    _: L,
  ) -> crate::Result<L>
  where
    L: LinearStorageLen,
  {
    Ok(L::ZERO)
  }

  #[inline]
  fn reset(&mut self) {}
}

impl<T> Decompression for Option<T>
where
  T: Decompression,
{
  const IS_NOOP: bool = T::IS_NOOP;

  #[inline]
  fn decompress<L>(
    &mut self,
    flush: DecompressionFlush,
    input: &[u8],
    output: &mut Vector<L, u8>,
    upper_bound: L,
  ) -> crate::Result<L>
  where
    L: LinearStorageLen,
  {
    if let Some(elem) = self {
      elem.decompress(flush, input, output, upper_bound)
    } else {
      Ok(L::ZERO)
    }
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
    codec::{CodecError, Decompression, DecompressionFlush},
    collections::{LinearStorageLen, Vector},
  };
  use zlib_rs::{Inflate, InflateFlush, Status};

  impl Decompression for Inflate {
    #[inline]
    fn decompress<L>(
      &mut self,
      flush: DecompressionFlush,
      input: &[u8],
      output: &mut Vector<L, u8>,
      upper_bound: L,
    ) -> crate::Result<L>
    where
      L: LinearStorageLen,
    {
      let additional = input.len().max(64);
      if output.len().usize().saturating_add(additional) > upper_bound.usize() {
        return Err(CodecError::DecompressionOverflow.into());
      }
      output.reserve(L::from_usize(additional)?)?;
      let mut curr_input = input;
      let mut total_read: L = L::ZERO;
      loop {
        let (_, uninit) = output.split_at_spare_mut();
        let uninit_len = uninit.len();
        let before_in = self.total_in();
        let before_out = self.total_out();
        let status = (*self).decompress_uninit(curr_input, uninit, flush.into())?;
        let consumed = self.total_in().wrapping_sub(before_in).try_into()?;
        let written = self.total_out().wrapping_sub(before_out);
        let written_usize: usize = written.try_into()?;
        let written_lsl = L::from_usize(written_usize)?;
        curr_input = curr_input.get(consumed..).unwrap_or_default();
        total_read = total_read.wrapping_add(written_lsl);
        let new_len = output.len().wrapping_add(written_lsl);
        // SAFETY: `decompress_uninit` just initialized `written` bytes
        unsafe {
          output.set_len(new_len);
        }
        match status {
          Status::BufError if curr_input.is_empty() => {
            return Ok(total_read);
          }
          Status::Ok if curr_input.is_empty() && written_usize < uninit_len => {
            return Ok(total_read);
          }
          Status::StreamEnd => {
            return Ok(total_read);
          }
          Status::BufError | Status::Ok => {}
        }
        {
          let local_additional = L::from_usize((output.capacity().usize() / 2).max(64))?;
          if output.len().usize().saturating_add(local_additional.usize()) > upper_bound.usize() {
            return Err(CodecError::DecompressionOverflow.into());
          }
          output.reserve(local_additional)?;
        }
      }
    }

    #[inline]
    fn reset(&mut self) {
      (*self).reset(false);
    }
  }

  impl From<DecompressionFlush> for InflateFlush {
    #[inline]
    fn from(value: DecompressionFlush) -> Self {
      match value {
        DecompressionFlush::NoFlush => InflateFlush::NoFlush,
        DecompressionFlush::SyncFlush => InflateFlush::SyncFlush,
      }
    }
  }
}
