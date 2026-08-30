use crate::{
  collections::Vector,
  http::HttpError,
  misc::{Lease, LeaseMut, SensitiveBytes, TryArithmetic as _},
};
use core::{
  fmt::{Arguments, Debug, Formatter},
  hint::unreachable_unchecked,
  str,
};

const METADATA_LEN: usize = 3;

/// Tells how trailers are placed in the headers
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Trailers {
  /// Does not have trailers
  None,
  /// Trailers are arbitrary placed inside the headers
  Mixed,
  /// All trailers are positioned at the end of the headers
  Tail(u16),
}

impl Trailers {
  /// If there is at least one trailer header, of any type.
  #[inline]
  pub const fn has_any(self) -> bool {
    matches!(self, Trailers::Mixed | Trailers::Tail(_))
  }
}

/// List of pairs sent and received on every request/response.
///
/// Internal operations are usually faster without sensitive content or trailers. If trailers
/// are necessary, then they should be preferably placed at the end.
pub struct Headers {
  bytes: Vector<u8>,
  headers: u16,
  sensitive_headers: u16,
  trailers: Trailers,
}

impl Headers {
  /// Empty instance
  #[inline]
  pub const fn new() -> Self {
    Self { bytes: Vector::new(), headers: 0, sensitive_headers: 0, trailers: Trailers::None }
  }

  /// Pre-allocates bytes according to the number of passed elements.
  ///
  /// Bytes are capped according to the specified `max_bytes`.
  #[inline]
  pub fn with_capacity(cap: usize) -> crate::Result<Self> {
    Ok(Self {
      bytes: Vector::with_capacity(cap)?,
      headers: 0,
      sensitive_headers: 0,
      trailers: Trailers::None,
    })
  }

  /// The amount of bytes used by all of the headers
  #[inline]
  pub fn bytes_len(&self) -> usize {
    self.bytes.len()
  }

  /// Clears the internal buffer "erasing" all previously inserted elements.
  ///
  /// ```rust
  /// use wtx::http::{Header, Headers};
  /// let mut headers = Headers::new();
  /// headers.push_from_iter(Header::from_name_and_value("name", ["value"])).unwrap();
  /// assert_eq!(headers.bytes_len(), 14);
  /// assert_eq!(headers.headers_len(), 1);
  /// headers.clear();
  /// assert_eq!(headers.bytes_len(), 0);
  /// assert_eq!(headers.headers_len(), 0);
  /// ```
  #[inline]
  pub fn clear(&mut self) {
    self.manage_erasing();
    let Self { bytes, headers, sensitive_headers, trailers } = self;
    bytes.clear();
    *headers = 0;
    *sensitive_headers = 0;
    *trailers = Trailers::None;
  }

  /// Returns the header that is referenced by `name`, if any.
  #[inline]
  pub fn get_by_name(&self, name: &[u8]) -> Option<Header<&str, &str>> {
    self.iter().find(|el| el.name.as_bytes() == name)
  }

  /// Returns all last optional headers that are referenced by `names`.
  ///
  /// ```rust
  /// use wtx::http::{Header, Headers};
  /// let mut headers = Headers::new();
  /// headers.push_from_iter(Header::from_name_and_value("name0", [])).unwrap();
  /// let array = headers.get_by_names([b"name0", b"name1"]);
  /// assert!(array[0].is_some());
  /// assert!(array[1].is_none());
  /// ```
  #[inline]
  pub fn get_by_names<const N: usize>(&self, names: [&[u8]; N]) -> [Option<Header<&str, &str>>; N] {
    let mut counter: usize = 0;
    let mut rslt = [None; N];
    for header in self.iter().rev() {
      if counter == N {
        break;
      }
      for (name, opt) in names.into_iter().zip(&mut rslt) {
        if opt.is_none() && name == header.name.as_bytes() {
          *opt = Some(header);
          counter = counter.wrapping_add(1);
          break;
        }
      }
    }
    rslt
  }

  /// The number of headers
  #[inline]
  pub fn headers_len(&self) -> u16 {
    self.headers
  }

  /// Retrieves all stored pairs.
  #[inline]
  pub fn iter(&self) -> HeadersIter<'_> {
    HeadersIter { bytes: &self.bytes }
  }

  /// Removes the last element.
  ///
  /// ```rust
  /// use wtx::http::{Header, Headers};
  /// let mut headers = Headers::new();
  /// headers.push_from_iter(Header::from_name_and_value("name", ["value"])).unwrap();
  /// assert_eq!(headers.bytes_len(), 14);
  /// assert_eq!(headers.headers_len(), 1);
  /// let _ = headers.pop();
  /// assert_eq!(headers.bytes_len(), 0);
  /// assert_eq!(headers.headers_len(), 0);
  /// ```
  #[inline]
  pub fn pop(&mut self) -> Option<()> {
    let [.., b0, b1] = self.bytes.as_slice() else {
      return None;
    };
    let begin_idx = u16::from_be_bytes([*b0, *b1]);
    let last_header = self.bytes.get_mut(begin_idx.into()..)?;
    let (mut header, _) = decode_header_mut(last_header)?;
    Self::manage_sensitive_content_deletion(&mut header, &mut self.sensitive_headers);
    self.manage_trailers_deletion(begin_idx);
    self.headers = self.headers.wrapping_sub(1);
    self.bytes.truncate(begin_idx.into());
    Some(())
  }

  /// Pushes a new header with its value composed by [`Arguments`].
  ///
  /// ```rust
  /// use wtx::http::{Header, Headers};
  /// let mut headers = Headers::new();
  /// headers.push_from_fmt(Header::from_name_and_value("name", format_args!("{}", 1))).unwrap();
  /// assert_eq!(headers.iter().next().unwrap(), Header::from_name_and_value("name", "1"));
  /// ```
  #[inline(always)]
  pub fn push_from_fmt(&mut self, header: Header<&str, Arguments<'_>>) -> crate::Result<()> {
    let begin_idx: u16 = self.bytes.len().try_into()?;
    let hm = HeaderMetadata::from_header(&header.strip_value())?;
    let mut write_fun = || {
      let _ = self.bytes.extend_from_copyable_slices([&hm.0, header.name.as_bytes()])?;
      let before_idx = self.bytes.len();
      cfg_select! {
        feature = "std" => {
          use std::io::Write as _;
          self.bytes.write_fmt(format_args!("{}", header.value))?;
        }
        _ => {
          use core::fmt::Write as _;
          self.bytes.write_fmt(format_args!("{}", header.value))?;
        }
      };
      self.adjust_after_write(begin_idx, before_idx)?;
      crate::Result::Ok(())
    };
    if let Err(err) = write_fun() {
      self.bytes.truncate(begin_idx.into());
      return Err(err);
    }
    self.manage_sensitive_content_inclusion(header.is_sensitive);
    self.manage_trailers_inclusion(header.is_trailer, begin_idx);
    self.headers = self.headers.wrapping_add(1);
    Ok(())
  }

  /// Pushes a new header with its value composed by several slices.
  ///
  /// ```rust
  /// use wtx::http::{Header, Headers};
  /// let mut headers = Headers::new();
  /// headers.push_from_iter(Header::from_name_and_value("name", ["value0", "_value1"])).unwrap();
  /// assert_eq!(headers.iter().next().unwrap(), Header::from_name_and_value("name", "value0_value1"));
  /// ```
  #[inline(always)]
  pub fn push_from_iter<'kv, V>(&mut self, header: Header<&'kv str, V>) -> crate::Result<()>
  where
    V: IntoIterator<Item = &'kv str>,
    V::IntoIter: Clone,
  {
    let begin_idx: u16 = self.bytes.len().try_into()?;
    let hm = HeaderMetadata::from_header(&header.strip_value())?;
    let iter = header.value.into_iter();
    let (additional, _) = Self::encoded_header_len(header.name, iter.clone())?;
    self.reserve(additional)?;
    let write_fun = || {
      let _ = self.bytes.extend_from_copyable_slices([hm.0.as_slice(), header.name.as_bytes()])?;
      let before_idx = self.bytes.len();
      for chunk in iter {
        self.bytes.extend_from_copyable_slice(chunk.as_bytes())?;
      }
      self.adjust_after_write(begin_idx, before_idx)?;
      crate::Result::Ok(())
    };
    if let Err(err) = write_fun() {
      self.bytes.truncate(begin_idx.into());
      return Err(err);
    }
    self.manage_sensitive_content_inclusion(header.is_sensitive);
    self.manage_trailers_inclusion(header.is_trailer, begin_idx);
    self.headers = self.headers.wrapping_add(1);
    Ok(())
  }

  /// Similarly to [`Self::push_from_iter`], pushes several headers.
  #[inline]
  pub fn push_from_iter_many<'bytes, const N: usize, V>(
    &mut self,
    headers: [Header<&'bytes str, V>; N],
  ) -> crate::Result<()>
  where
    V: Clone + Iterator<Item = &'bytes str>,
  {
    let mut header_len: usize = 0;
    for header in &headers {
      let (additional, _) = Self::encoded_header_len(header.name, header.value.clone())?;
      header_len = header_len.wrapping_add(additional);
    }
    self.reserve(header_len)?;
    for header in headers {
      self.push_from_iter(header)?;
    }
    Ok(())
  }

  /// Reserves capacity for at least `cap` more bytes to be inserted.
  #[inline(always)]
  pub fn reserve(&mut self, additional: usize) -> crate::Result<()> {
    self.bytes.reserve(additional)?;
    Ok(())
  }

  /// If this instance has one or more trailer headers.
  #[inline]
  pub const fn trailers(&self) -> Trailers {
    self.trailers
  }

  #[inline]
  fn adjust_after_write(&mut self, begin_idx: u16, before_idx: usize) -> Result<(), crate::Error> {
    let after_idx = self.bytes.len();
    let value_len: u16 = after_idx.wrapping_sub(before_idx).try_into()?;
    if let Some([_, b1, b2, ..]) = self.bytes.get_mut(usize::from(begin_idx)..) {
      let [b3, b4] = value_len.to_be_bytes();
      *b1 = b3;
      *b2 = b4;
    }
    self.bytes.extend_from_copyable_slice(&begin_idx.to_be_bytes())?;
    Ok(())
  }

  #[inline]
  fn encoded_header_len<'bytes>(
    header_name: &str,
    iter: impl Iterator<Item = &'bytes str>,
  ) -> crate::Result<(usize, u16)> {
    let mut value_len: u16 = 0;
    for elem in iter {
      value_len = value_len.try_add(elem.len().try_into()?)?;
    }
    Ok((5usize.wrapping_add(header_name.len()).wrapping_add(value_len.into()), value_len))
  }

  #[inline]
  fn manage_erasing(&mut self) {
    if self.sensitive_headers == 0 {
      return;
    }
    let mut local_bytes = &mut *self.bytes;
    while let Some((hm_bytes, rest0)) = local_bytes.split_first_chunk_mut::<METADATA_LEN>() {
      let hm = HeaderMetadata(*hm_bytes);
      // SAFETY: Created headers always respect the associated lengths
      let (_, rest1) = unsafe { rest0.split_at_mut_unchecked(hm.name_len().into()) };
      let Some((value, [_, _, rest2 @ ..])) = rest1.split_at_mut_checked(hm.value_len().into())
      else {
        // SAFETY: Created headers always respect the associated lengths
        unsafe {
          unreachable_unchecked();
        }
      };
      if hm.is_sensitive() {
        drop(SensitiveBytes::new(value));
      }
      local_bytes = rest2;
    }
  }

  #[inline]
  fn manage_sensitive_content_deletion(
    header: &mut Header<&mut str, &mut str>,
    sensitive_headers: &mut u16,
  ) {
    if header.is_sensitive {
      *sensitive_headers = sensitive_headers.wrapping_sub(1);
      // SAFETY: Zeros are ASCII
      drop(SensitiveBytes::new(unsafe { header.value.as_bytes_mut() }));
    }
  }

  #[inline]
  fn manage_sensitive_content_inclusion(&mut self, is_sensitive: bool) {
    self.sensitive_headers = self.sensitive_headers.wrapping_add(is_sensitive.into());
  }

  #[inline]
  fn manage_trailers_deletion(&mut self, popped_idx: u16) {
    if popped_idx == 0 {
      self.trailers = Trailers::None;
      return;
    }
    match self.trailers {
      Trailers::Tail(idx) if idx == popped_idx => {
        self.trailers = Trailers::None;
      }
      Trailers::Mixed | Trailers::None | Trailers::Tail(_) => {}
    }
  }

  #[inline]
  const fn manage_trailers_inclusion(&mut self, is_trailer: bool, prev_len: u16) {
    self.trailers = if is_trailer {
      match self.trailers {
        Trailers::Mixed => Trailers::Mixed,
        Trailers::None => Trailers::Tail(prev_len),
        Trailers::Tail(idx) => Trailers::Tail(idx),
      }
    } else {
      match self.trailers {
        Trailers::Mixed | Trailers::Tail(_) => Trailers::Mixed,
        Trailers::None => Trailers::None,
      }
    };
  }
}

impl Lease<Headers> for Headers {
  #[inline]
  fn lease(&self) -> &Headers {
    self
  }
}

impl LeaseMut<Headers> for Headers {
  #[inline]
  fn lease_mut(&mut self) -> &mut Headers {
    self
  }
}

impl Debug for Headers {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_list().entries(self.iter()).finish()
  }
}

impl Default for Headers {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl Drop for Headers {
  #[inline]
  fn drop(&mut self) {
    self.manage_erasing();
  }
}

impl<'any> IntoIterator for &'any Headers {
  type Item = Header<&'any str, &'any str>;
  type IntoIter = HeadersIter<'any>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

/// A field of an HTTP request or response.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header<N, V> {
  /// If the name/value should NOT be cached.
  ///
  /// The applicability of this parameter depends on the HTTP version.
  pub is_sensitive: bool,
  /// Trailers are added at the end of a message.
  ///
  /// The applicability and semantics depends on the HTTP version.
  pub is_trailer: bool,
  /// Header name
  pub name: N,
  /// Header value
  pub value: V,
}

impl<N, V> Header<N, V> {
  /// Constructor shortcut
  #[inline]
  pub const fn new(is_sensitive: bool, is_trailer: bool, name: N, value: V) -> Self {
    Self { is_sensitive, is_trailer, name, value }
  }

  /// Sets `is_sensitive` and `is_trailer` to `false`.
  #[inline]
  pub const fn from_name_and_value(name: N, value: V) -> Self {
    Self { is_sensitive: false, is_trailer: false, name, value }
  }
}

impl<'any, V> Header<&'any str, V> {
  #[inline]
  fn strip_value(&self) -> Header<&'any str, &'any str> {
    Header {
      is_sensitive: self.is_sensitive,
      is_trailer: self.is_trailer,
      name: self.name,
      value: "",
    }
  }
}

// ```
// | Header Metadata                                              |
// | Header length | Is Sensitive | Is Trailer | Value length     |
// | xxxxxx        | x            | x          | xxxxxxxxxxxxxxxx |
// ```
#[derive(Debug)]
struct HeaderMetadata([u8; METADATA_LEN]);

impl HeaderMetadata {
  fn from_header(header: &Header<&str, &str>) -> crate::Result<Self> {
    let name_len: u8 = header.name.len().try_into()?;
    if name_len >= 64 {
      return Err(HttpError::LargeHeaderName.into());
    }
    let value_len: u16 = header.value.len().try_into()?;
    let is_sensitive = if header.is_sensitive { 0b0000_0010 } else { 0 };
    let is_trailer = u8::from(header.is_trailer);
    let b0 = (name_len << 2) | is_sensitive | is_trailer;
    let [b1, b2] = value_len.to_be_bytes();
    Ok(Self([b0, b1, b2]))
  }

  const fn is_sensitive(&self) -> bool {
    self.0[0] & 0b0000_0010 != 0
  }

  const fn is_trailer(&self) -> bool {
    self.0[0] & 0b0000_0001 != 0
  }

  const fn name_len(&self) -> u8 {
    self.0[0] >> 2
  }

  const fn value_len(&self) -> u16 {
    let [_, b1, b2] = self.0;
    u16::from_be_bytes([b1, b2])
  }
}

/// Iterator of the [`Headers::iter`] method
#[derive(Debug)]
pub struct HeadersIter<'any> {
  bytes: &'any [u8],
}

impl<'any> Iterator for HeadersIter<'any> {
  type Item = Header<&'any str, &'any str>;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    let (header, rest) = decode_header(self.bytes)?;
    self.bytes = rest;
    Some(header)
  }
}

impl DoubleEndedIterator for HeadersIter<'_> {
  #[inline]
  fn next_back(&mut self) -> Option<Self::Item> {
    let [.., b0, b1] = self.bytes else {
      return None;
    };
    let begin_idx = u16::from_be_bytes([*b0, *b1]);
    let (lhs, rhs) = self.bytes.split_at_checked(begin_idx.into())?;
    let (header, _) = decode_header(rhs)?;
    self.bytes = lhs;
    Some(header)
  }
}

#[inline(always)]
fn decode_header(bytes: &[u8]) -> Option<(Header<&str, &str>, &[u8])> {
  let (hm_bytes, rest0) = bytes.split_first_chunk::<METADATA_LEN>()?;
  let hm = HeaderMetadata(*hm_bytes);
  // SAFETY: Created headers always respect the associated lengths
  let (name, rest1) = unsafe { rest0.split_at_unchecked(hm.name_len().into()) };
  let Some((value, [_, _, rest2 @ ..])) = rest1.split_at_checked(hm.value_len().into()) else {
    // SAFETY: Created headers always respect the associated lengths
    unsafe {
      unreachable_unchecked();
    }
  };
  let header = Header {
    is_sensitive: hm.is_sensitive(),
    is_trailer: hm.is_trailer(),
    // SAFETY: input methods only accept UTF-8 data
    name: unsafe { str::from_utf8_unchecked(name) },
    // SAFETY: input methods only accept UTF-8 data
    value: unsafe { str::from_utf8_unchecked(value) },
  };
  Some((header, rest2))
}

#[inline(always)]
fn decode_header_mut(bytes: &mut [u8]) -> Option<(Header<&mut str, &mut str>, &mut [u8])> {
  let (hm_bytes, rest0) = bytes.split_first_chunk_mut::<METADATA_LEN>()?;
  let hm = HeaderMetadata(*hm_bytes);
  // SAFETY: Created headers always respect the associated lengths
  let (name, rest1) = unsafe { rest0.split_at_mut_unchecked(hm.name_len().into()) };
  let Some((value, [_, _, rest2 @ ..])) = rest1.split_at_mut_checked(hm.value_len().into()) else {
    // SAFETY: Created headers always respect the associated lengths
    unsafe {
      unreachable_unchecked();
    }
  };
  let header = Header {
    is_sensitive: hm.is_sensitive(),
    is_trailer: hm.is_trailer(),
    // SAFETY: input methods only accept UTF-8 data
    name: unsafe { str::from_utf8_unchecked_mut(name) },
    // SAFETY: input methods only accept UTF-8 data
    value: unsafe { str::from_utf8_unchecked_mut(value) },
  };
  Some((header, rest2))
}

#[cfg(test)]
mod tests {
  use crate::http::{Header, Headers, KnownHeaderName, Trailers};

  #[test]
  fn pop_resets_trailer_tail_state_correctly() {
    let mut headers = Headers::new();

    headers
      .push_from_iter(Header::from_name_and_value(
        KnownHeaderName::ContentType.into(),
        ["text/plain"],
      ))
      .unwrap();
    assert_eq!(headers.bytes_len(), 27);
    assert_eq!(headers.headers_len(), 1);
    assert_eq!(headers.trailers(), Trailers::None);

    headers.push_from_iter(Header::new(false, true, "x-trailer-a", ["value"])).unwrap();
    assert_eq!(headers.bytes_len(), 27 + 21);
    assert_eq!(headers.headers_len(), 2);
    assert_eq!(headers.trailers(), Trailers::Tail(27));

    assert!(headers.pop().is_some());

    assert_eq!(headers.bytes_len(), 27);
    assert_eq!(headers.headers_len(), 1);
    assert_eq!(headers.trailers(), Trailers::None);

    let first = headers.get_by_name(<&[u8]>::from(KnownHeaderName::ContentType)).unwrap();
    assert!(!first.is_trailer);
  }
}
