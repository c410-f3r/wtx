use crate::{
  collection::{IndexedStorageMut as _, Vector},
  misc::bytes_split1,
};
#[cfg(feature = "std")]
use std::io::BufRead;

/// Simple reader of CSV files
#[derive(Debug)]
pub struct Csv<R, S> {
  delimiter: u8,
  reader: R,
  source: S,
}

impl<R, S> Csv<R, S>
where
  R: FnMut(&mut Vector<u8>, &mut S) -> crate::Result<usize>,
{
  /// New instance
  #[inline]
  pub const fn new(delimiter: u8, reader: R, source: S) -> Self {
    Self { delimiter, reader, source }
  }

  /// Reads the next line from the source and returns an iterator over the fields.
  #[inline]
  pub fn next_line<'buffer>(
    &mut self,
    buffer: &'buffer mut Vector<u8>,
  ) -> crate::Result<Option<impl Iterator<Item = &'buffer [u8]>>> {
    buffer.clear();
    let line_len = (self.reader)(buffer, &mut self.source)?;
    if line_len == 0 {
      return Ok(None);
    }
    let line = buffer.get(..line_len).unwrap_or_default();
    Ok(Some(bytes_split1(line, self.delimiter)))
  }
}

#[cfg(feature = "std")]
impl<S> Csv<fn(&mut Vector<u8>, &mut S) -> crate::Result<usize>, S>
where
  S: BufRead,
{
  /// New instance based on [`BufRead`].
  #[inline]
  pub const fn from_buf_read(delimiter: u8, source: S) -> Self {
    fn reader<S>(buffer: &mut Vector<u8>, source: &mut S) -> crate::Result<usize>
    where
      S: BufRead,
    {
      Ok(source.read_until(b'\n', buffer.as_vec_mut())?)
    }
    Self { delimiter, reader, source }
  }
}
