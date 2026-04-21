use crate::{
  codec::CodecError,
  collection::{ArrayVectorU8, Clear},
  misc::{Lease, bytes_split2_indices, strip_new_line},
};
use core::{marker::PhantomData, ops::Range};
#[cfg(feature = "std")]
use {crate::collection::Vector, std::io::BufRead};

const MAX_COLUMNS: usize = 64;

/// Reader of CSV data according to RFC-4180.
///
/// The only exception is the use of double quotes inside quoted fields because such a scenario
/// will raise an error.
///
/// <https://datatracker.ietf.org/doc/html/rfc4180>
#[derive(Debug)]
pub struct Csv<B, R, S>
where
  R: FnMut(&mut B, &mut S) -> crate::Result<usize>,
{
  indices: ArrayVectorU8<Range<u16>, MAX_COLUMNS>,
  phantom: PhantomData<B>,
  reader: R,
  source: S,
}

impl<B, R, S> Csv<B, R, S>
where
  B: Clear + Lease<[u8]>,
  R: FnMut(&mut B, &mut S) -> crate::Result<usize>,
{
  /// New instance
  #[inline]
  pub const fn new(reader: R, source: S) -> Self {
    Self { indices: ArrayVectorU8::new(), phantom: PhantomData, reader, source }
  }

  /// Reads the next line from the source and returns an iterator over the fields.
  #[inline]
  pub fn next_elements<'eb>(
    &mut self,
    elements_buffer: &'eb mut B,
  ) -> crate::Result<Option<impl Iterator<Item = &'eb [u8]>>>
  where
    B: Clear + Lease<[u8]>,
  {
    elements_buffer.clear();
    self.indices.clear();

    let is_in_quote = &mut false;
    let is_quoted = &mut false;
    let prev_idx = &mut 0u16;
    let scan_from = &mut 0usize;

    loop {
      let local_line_len: u16 = (self.reader)(elements_buffer, &mut self.source)?
        .try_into()
        .map_err(|_err| CodecError::CsvLineOverflow)?;
      if local_line_len == 0 {
        break;
      }
      let (suffixes, line) = strip_new_line(elements_buffer.lease());
      if line.len() > u16::MAX.into() {
        return Err(CodecError::CsvLineOverflow.into());
      }

      let iter_line = line.get(*scan_from..).unwrap_or_default();
      let iter = &mut bytes_split2_indices(iter_line, [b',', b'"']);
      let mut has_init = *is_in_quote;

      self.eval_first(&mut has_init, is_in_quote, is_quoted, iter, line, prev_idx, *scan_from)?;
      self.eval_middle(is_in_quote, is_quoted, iter, line, prev_idx, scan_from)?;
      self.eval_last(has_init, is_in_quote, is_quoted, line, prev_idx)?;

      match (suffixes == 0, *is_in_quote) {
        (true, true) if line.last().copied() != Some(b'"') => {
          return Err(CodecError::CsvInvalidQuotes.into());
        }
        (false, false) => {
          break;
        }
        _ => {}
      }
    }

    if self.indices.is_empty() {
      return Ok(None);
    }

    Ok(Some(ElementsIter { buffer: elements_buffer, idx: 0, indices: &self.indices }))
  }

  #[inline]
  fn eval_first(
    &mut self,
    has_init: &mut bool,
    is_in_quote: &mut bool,
    is_quoted: &mut bool,
    iter: &mut impl Iterator<Item = usize>,
    line: &[u8],
    prev_idx: &mut u16,
    scan_from: usize,
  ) -> crate::Result<()> {
    if *has_init {
      return Ok(());
    }
    for idx in iter.by_ref() {
      let abs_idx = idx16(scan_from.wrapping_add(idx));
      *has_init = self.manage_field::<true>(abs_idx, is_in_quote, is_quoted, line, prev_idx)?;
      if *has_init {
        break;
      }
    }
    Ok(())
  }

  #[inline]
  fn eval_last(
    &mut self,
    has_init: bool,
    is_in_quote: &mut bool,
    is_quoted: &mut bool,
    line: &[u8],
    prev_idx: &mut u16,
  ) -> crate::Result<()> {
    if *is_in_quote {
      return Ok(());
    }
    if has_init {
      self.push_field::<false>(idx16(line.len()), is_in_quote, *is_quoted, prev_idx)?;
    } else {
      self.push_field::<true>(idx16(line.len()), is_in_quote, *is_quoted, prev_idx)?;
    }
    Ok(())
  }

  #[inline]
  fn eval_middle(
    &mut self,
    is_in_quote: &mut bool,
    is_quoted: &mut bool,
    iter: &mut impl Iterator<Item = usize>,
    line: &[u8],
    prev_idx: &mut u16,
    scan_from: &mut usize,
  ) -> crate::Result<()> {
    for idx in iter {
      let abs_idx = idx16(scan_from.wrapping_add(idx));
      let _ = self.manage_field::<false>(abs_idx, is_in_quote, is_quoted, line, prev_idx)?;
    }
    *scan_from = line.len();
    Ok(())
  }

  // The RFC states that quoted fields must not have empty spaces between delimiters and spaces
  // of unquoted fields must be preserved.
  //
  // `is_in_quote`: If the cursor is currently constructing an unfinished quoted field.
  // `is_quoted`: If the most recently constructed field is quoted.
  #[inline]
  fn manage_field<const IS_FIRST: bool>(
    &mut self,
    curr_idx: u16,
    is_in_quote: &mut bool,
    is_quoted: &mut bool,
    line: &[u8],
    prev_idx: &mut u16,
  ) -> crate::Result<bool> {
    let uidx = usize::from(curr_idx);
    match line.get(uidx).copied() {
      Some(b'"') => {
        if *is_in_quote {
          let next = line.get(uidx.wrapping_add(1)).copied();
          if next.is_some() && next != Some(b',') {
            return Err(CodecError::CsvInvalidQuotes.into());
          }
          *is_in_quote = false;
        } else {
          let field_begin_idx = if IS_FIRST { *prev_idx } else { prev_idx.wrapping_add(1) };
          if curr_idx == field_begin_idx {
            *is_in_quote = true;
            *is_quoted = true;
          } else {
            return Err(CodecError::CsvInvalidQuotes.into());
          }
        }
      }
      Some(b',') if !*is_in_quote => {
        self.push_field::<IS_FIRST>(curr_idx, is_in_quote, *is_quoted, prev_idx)?;
        *is_quoted = false;
        *prev_idx = curr_idx;
        return Ok(true);
      }
      _ => {}
    }
    Ok(false)
  }

  #[inline]
  fn push_field<const IS_FIRST: bool>(
    &mut self,
    curr_idx: u16,
    is_in_quote: &mut bool,
    is_quoted: bool,
    prev_idx: &mut u16,
  ) -> crate::Result<()> {
    if is_quoted {
      // `" data ",` or `," data ",` or `," data "`
      let begin = prev_idx.wrapping_add(if IS_FIRST { 1 } else { 2 });
      let end = curr_idx.wrapping_sub(1);
      self.indices.push(begin..end)?;
      *is_in_quote = false;
    } else {
      // ` data ,` or `, data ,`  or `, data `
      let begin = prev_idx.wrapping_add(u16::from(!IS_FIRST));
      let end = curr_idx;
      self.indices.push(begin..end)?;
    }
    Ok(())
  }
}

#[cfg(feature = "std")]
impl<S> Csv<Vector<u8>, fn(&mut Vector<u8>, &mut S) -> crate::Result<usize>, S>
where
  S: BufRead,
{
  /// New instance based on [`BufRead`].
  #[inline]
  pub const fn from_buf_read(source: S) -> Self {
    fn reader<S>(buffer: &mut Vector<u8>, source: &mut S) -> crate::Result<usize>
    where
      S: BufRead,
    {
      Ok(source.read_until(b'\n', buffer.vec_mut())?)
    }
    Self { indices: ArrayVectorU8::new(), phantom: PhantomData, reader, source }
  }
}

struct ElementsIter<'buffer, 'instance, B> {
  buffer: &'buffer B,
  idx: u16,
  indices: &'instance ArrayVectorU8<Range<u16>, MAX_COLUMNS>,
}

impl<'buffer, B> Iterator for ElementsIter<'buffer, '_, B>
where
  B: Lease<[u8]>,
{
  type Item = &'buffer [u8];

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    let range = self.indices.get(usize::from(self.idx))?;
    self.idx = self.idx.wrapping_add(1);
    self.buffer.lease().get(range.start.into()..range.end.into())
  }
}

#[expect(clippy::cast_possible_truncation, reason = "all callers don't exceed 65_535 ")]
#[inline]
const fn idx16(idx: usize) -> u16 {
  idx as u16
}

#[cfg(test)]
mod tests {
  use crate::{
    codec::Csv,
    collection::{ArrayVectorU8, Vector},
  };
  use core::str;
  use std::io::BufReader;

  #[test]
  fn double_quote() {
    assert!(parse_one(&mut Vector::new(), "\"aaa\"\"aaa\"").is_none());
    assert!(parse_one(&mut Vector::new(), "aaa\"\"aaa").is_none());
  }

  #[test]
  fn empty_fields_and_quotes() {
    assert_eq!(parse_one(&mut Vector::new(), ",,").unwrap(), &["", "", ""]);
    assert_eq!(parse_one(&mut Vector::new(), "a,\"\",c").unwrap(), &["a", "", "c"]);
    assert_eq!(parse_one(&mut Vector::new(), "\"\"").unwrap(), &[""]);
  }

  #[test]
  fn multiline() {
    let data = "1,\"hello\nworld\",2";
    let mut line_buffer = Vector::new();
    let mut csv = Csv::from_buf_read(BufReader::new(data.as_bytes()));
    {
      let mut fields = csv.next_elements(&mut line_buffer).unwrap().unwrap();
      assert_eq!(fields.next().unwrap(), b"1");
      assert_eq!(fields.next().unwrap(), b"hello\nworld");
      assert_eq!(fields.next().unwrap(), b"2");
      assert!(fields.next().is_none());
    }
    assert!(csv.next_elements(&mut line_buffer).unwrap().is_none());
  }

  #[test]
  fn multiline_many() {
    let data = "\"a\",\"b\"\n\"aaaa\naaaa\",\"aaaa\"\n\"aaaa\",\"aaaa\"\n\"aaaa\",\"aaaa\naaaa\"";
    let mut line_buffer = Vector::new();
    let mut csv = Csv::from_buf_read(BufReader::new(data.as_bytes()));
    assert_eq!(csv.next_elements(&mut line_buffer).unwrap().unwrap().count(), 2);
    assert_eq!(csv.next_elements(&mut line_buffer).unwrap().unwrap().count(), 2);
    assert_eq!(csv.next_elements(&mut line_buffer).unwrap().unwrap().count(), 2);
    assert_eq!(csv.next_elements(&mut line_buffer).unwrap().unwrap().count(), 2);
    assert!(csv.next_elements(&mut line_buffer).unwrap().is_none());
  }

  #[test]
  fn quote_followed_by_non_delimiter() {
    assert!(parse_one(&mut Vector::new(), "\"a\"b").is_none());
    assert!(parse_one(&mut Vector::new(), "\"a\"x,c").is_none());
    assert!(parse_one(&mut Vector::new(), "a,\"b\"c").is_none());
    assert!(parse_one(&mut Vector::new(), "\"a\" ").is_none());
  }

  #[test]
  fn single_field() {
    assert_eq!(parse_one(&mut Vector::new(), "a").unwrap(), &["a"]);
    assert_eq!(parse_one(&mut Vector::new(), "abc").unwrap(), &["abc"]);
    assert_eq!(parse_one(&mut Vector::new(), "\"a\"").unwrap(), &["a"]);
    assert_eq!(parse_one(&mut Vector::new(), "\"abc\"").unwrap(), &["abc"]);
  }

  #[test]
  fn single_line() {
    assert_eq!(parse_one(&mut Vector::new(), "a,b,c,d,e").unwrap(), &["a", "b", "c", "d", "e"]);
    assert_eq!(parse_one(&mut Vector::new(), "a,b,").unwrap(), &["a", "b", ""]);
    assert_eq!(parse_one(&mut Vector::new(), ",\n").unwrap(), &["", ""]);
    assert_eq!(parse_one(&mut Vector::new(), "a,b,\n").unwrap(), &["a", "b", ""]);
    assert_eq!(parse_one(&mut Vector::new(), ",b\n").unwrap(), &["", "b"]);
    assert_eq!(parse_one(&mut Vector::new(), "a,b").unwrap(), &["a", "b"]);
    assert_eq!(parse_one(&mut Vector::new(), "a,b,c\n").unwrap(), &["a", "b", "c"]);
    assert_eq!(parse_one(&mut Vector::new(), "a,b\r\n").unwrap(), &["a", "b"]);
    assert_eq!(parse_one(&mut Vector::new(), "\"a\",\"b,c\",\"d\"").unwrap(), &["a", "b,c", "d"]);
    assert_eq!(parse_one(&mut Vector::new(), "\"\",b").unwrap(), &["", "b"]);

    assert!(parse_one(&mut Vector::new(), "ab\"cd\",ef").is_none());
  }

  #[test]
  fn unclosed_quote() {
    assert!(parse_one(&mut Vector::new(), "\"a").is_none());
    assert!(parse_one(&mut Vector::new(), "a,\"b").is_none());
    assert!(parse_one(&mut Vector::new(), "\"abc,def").is_none());
  }

  fn parse_one<'buffer>(
    buffer: &'buffer mut Vector<u8>,
    data: &str,
  ) -> Option<ArrayVectorU8<&'buffer str, 8>> {
    let mut csv = Csv::from_buf_read(BufReader::new(data.as_bytes()));
    ArrayVectorU8::from_iterator(
      csv.next_elements(buffer).ok()??.map(|el| str::from_utf8(el).unwrap()),
    )
    .ok()
  }
}
