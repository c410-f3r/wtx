use crate::{
  http::{abstract_headers::AbstractHeader, AbstractHeaders},
  misc::{Lease, LeaseMut},
};

/// List of pairs sent and received on every request.
#[derive(Debug)]
pub struct Headers {
  ab: AbstractHeaders<()>,
}

impl Headers {
  /// Empty instance
  #[inline]
  pub const fn new(max_bytes: usize) -> Self {
    Self { ab: AbstractHeaders::new(max_bytes) }
  }

  /// Pre-allocates bytes according to the number of passed elements.
  ///
  /// Bytes are capped according to the specified `max_bytes`.
  #[inline]
  pub fn with_capacity(bytes: usize, headers: usize, max_bytes: usize) -> Self {
    Self { ab: AbstractHeaders::with_capacity(bytes, headers, max_bytes) }
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
  }

  /// The number of headers
  #[inline]
  pub fn elements_len(&self) -> usize {
    self.ab.headers_len()
  }

  /// Returns the first header, if any.
  #[inline]
  pub fn first(&self) -> Option<(&[u8], &[u8], bool)> {
    self.ab.first().map(Self::map)
  }

  /// Returns the header's pair referenced by its index, if any.
  #[inline]
  pub fn get_by_idx(&self, idx: usize) -> Option<(&[u8], &[u8], bool)> {
    self.ab.get_by_idx(idx).map(Self::map)
  }

  /// Retrieves all stored pairs.
  #[inline]
  pub fn iter(&self) -> impl Iterator<Item = (&[u8], &[u8], bool)> {
    self.ab.iter().map(Self::map)
  }

  /// Pushes a new pair of `name` and `value` at the end of the internal buffer.
  ///
  /// If the sum of `name` and `value` is greater than the maximum number of bytes, then the first
  /// inserted entries will be deleted accordantly.
  #[inline]
  pub fn last(&self) -> Option<(&[u8], &[u8], bool)> {
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
  #[inline]
  pub fn push_front(&mut self, name: &[u8], value: &[u8], is_sensitive: bool) -> crate::Result<()> {
    self.ab.push_front((), name, value, is_sensitive, |_, _| {})
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
  pub fn reserve(&mut self, bytes: usize, headers: usize) {
    self.ab.reserve(bytes, headers);
  }

  /// If `max_bytes` is lesser than the current number of bytes, then the first inserted entries
  /// will be deleted accordantly.
  #[inline]
  pub fn set_max_bytes(&mut self, max_bytes: usize) {
    self.ab.set_max_bytes(max_bytes, |_, _| {});
  }

  #[inline]
  fn map(elem: AbstractHeader<'_, ()>) -> (&[u8], &[u8], bool) {
    (elem.name_bytes, elem.value_bytes, elem.is_sensitive)
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
