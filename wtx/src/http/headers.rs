use crate::{
  http::{abstract_headers::AbstractHeader, AbstractHeaders},
  misc::{Lease, LeaseMut},
};

/// List of pairs sent and received on every request/response.
#[derive(Debug)]
pub struct Headers {
  ab: AbstractHeaders<bool>,
  has_trailers: bool,
}

impl Headers {
  /// Empty instance
  #[inline]
  pub const fn new(max_bytes: usize) -> Self {
    Self { ab: AbstractHeaders::new(max_bytes), has_trailers: false }
  }

  /// Pre-allocates bytes according to the number of passed elements.
  ///
  /// Bytes are capped according to the specified `max_bytes`.
  #[inline]
  pub fn with_capacity(bytes: usize, headers: usize, max_bytes: usize) -> crate::Result<Self> {
    Ok(Self { ab: AbstractHeaders::with_capacity(bytes, headers, max_bytes)?, has_trailers: false })
  }

  /// The amount of bytes used by all of the headers
  #[inline]
  pub fn bytes_len(&self) -> usize {
    self.ab.bytes_len()
  }

  /// Clears the internal buffer "erasing" all previously inserted elements.
  #[inline]
  pub fn clear(&mut self) {
    self.ab.clear();
    self.has_trailers = false;
  }

  /// The number of headers
  #[inline]
  pub fn elements_len(&self) -> usize {
    self.ab.headers_len()
  }

  /// Returns the first header, if any.
  #[inline]
  pub fn first(&self) -> Option<Header<'_>> {
    self.ab.first().map(Self::map)
  }

  /// Returns the header's pair referenced by its index, if any.
  #[inline]
  pub fn get_by_idx(&self, idx: usize) -> Option<Header<'_>> {
    self.ab.get_by_idx(idx).map(Self::map)
  }

  /// Returns the header's pair referenced by its name, if any.
  #[inline]
  pub fn get_by_name(&self, name: &[u8]) -> Option<Header<'_>> {
    self.ab.get_by_name(name).map(Self::map)
  }

  /// If this instance has one or more trailer headers.
  #[inline]
  pub fn has_trailers(&self) -> bool {
    self.has_trailers
  }

  /// Retrieves all stored pairs.
  #[inline]
  pub fn iter(&self) -> impl Iterator<Item = Header<'_>> {
    self.ab.iter().map(Self::map)
  }

  /// Pushes a new pair of `name` and `value` at the end of the internal buffer.
  ///
  /// If the sum of `name` and `value` is greater than the maximum number of bytes, then the first
  /// inserted entries will be deleted accordantly.
  #[inline]
  pub fn last(&self) -> Option<Header<'_>> {
    self.ab.last().map(Self::map)
  }

  /// The maximum allowed number of bytes.
  #[inline]
  pub fn max_bytes(&self) -> usize {
    self.ab.max_bytes()
  }

  /// Removes the last element.
  #[inline]
  pub fn pop_back(&mut self) {
    let _ = self.ab.pop_back();
  }

  /// Removes the first element.
  #[inline]
  pub fn pop_front(&mut self) {
    let _ = self.ab.pop_front();
  }

  /// Pushes a new pair of `name` and `value` at the beginning of the internal buffer.
  ///
  /// If the sum of `name` and `value` is greater than the maximum number of bytes, then the first
  /// inserted entries will be deleted accordantly.
  ///
  /// `additional_value` can be used to append more data into the header value.
  #[inline]
  pub fn push_front(&mut self, header: Header<'_>, additional_value: &[u8]) -> crate::Result<()> {
    self.has_trailers = header.is_trailer;
    self.ab.push_front(
      header.is_trailer,
      header.name,
      [header.value, additional_value],
      header.is_sensitive,
      |_, _| {},
    )
  }

  /// Removes all a pair referenced by `idx`.
  #[inline]
  pub fn remove_by_idx(&mut self, idx: usize) -> bool {
    self.ab.remove_by_idx(idx).is_some()
  }

  /// Reserves capacity for at least `bytes` more bytes to be inserted. The same thing is applied
  /// to the number of headers.
  ///
  /// Bytes are capped according to the specified `max_bytes`.
  #[inline(always)]
  pub fn reserve(&mut self, bytes: usize, headers: usize) -> crate::Result<()> {
    self.ab.reserve(bytes, headers)
  }

  /// If `max_bytes` is lesser than the current number of bytes, then the first inserted entries
  /// will be deleted accordantly.
  #[inline]
  pub fn set_max_bytes(&mut self, max_bytes: usize) {
    self.ab.set_max_bytes(max_bytes, |_, _| {});
  }

  #[inline]
  fn map(elem: AbstractHeader<'_, bool>) -> Header<'_> {
    Header {
      is_sensitive: elem.is_sensitive,
      is_trailer: *elem.misc,
      name: elem.name_bytes,
      value: elem.value_bytes,
    }
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

/// A field of an HTTP request or response.
#[derive(Clone, Copy, Debug)]
pub struct Header<'any> {
  /// If the name/value should NOT be cached.
  ///
  /// The applicability of this parameter depends on the HTTP version.
  pub is_sensitive: bool,
  /// Trailers are added at the end of a message.
  ///
  /// The applicability and semantics depends on the HTTP version.
  pub is_trailer: bool,
  /// Header name
  pub name: &'any [u8],
  /// Header value
  pub value: &'any [u8],
}

impl<'any> From<(&'any [u8], &'any [u8])> for Header<'any> {
  #[inline]
  fn from((name, value): (&'any [u8], &'any [u8])) -> Self {
    Self { is_sensitive: false, is_trailer: false, name, value }
  }
}

impl<'any, const N: usize> From<(&'any [u8; N], &'any [u8; N])> for Header<'any> {
  #[inline]
  fn from((name, value): (&'any [u8; N], &'any [u8; N])) -> Self {
    Self { is_sensitive: false, is_trailer: false, name, value }
  }
}
