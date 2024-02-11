use crate::http::AbstractHeaders;

/// List of pairs sent and received on every request.
#[derive(Debug, Default)]
pub struct Headers {
  ab: AbstractHeaders<()>,
}

impl Headers {
  /// Pre-allocates bytes according to the number of passed elements.
  #[inline]
  pub fn with_capacity(len: u32) -> Self {
    Self { ab: AbstractHeaders::with_capacity(len) }
  }

  /// The amount of bytes used by all of the headers
  #[inline]
  pub fn bytes_len(&self) -> u32 {
    self.ab.bytes_len()
  }

  /// Clears the internal buffer "erasing" all previously inserted elements.
  #[inline]
  pub fn clear(&mut self) {
    self.ab.clear();
  }

  /// The number of headers
  #[inline]
  pub fn elements_len(&self) -> u32 {
    self.ab.elements_len()
  }

  /// Returns the header's pair referenced by its index, if any.
  #[inline]
  pub fn get_by_idx(&self, idx: usize) -> Option<(&[u8], &[u8])> {
    self.ab.get_by_idx(idx).map(|el| (el.name_bytes, el.value_bytes))
  }

  /// Returns the header value of the **first** corresponding header `name` key, if any.
  #[inline]
  pub fn get_by_name(&self, name: &[u8]) -> Option<&[u8]> {
    self.ab.get_by_name(name).map(|el| el.value_bytes)
  }

  /// Retrieves all stored pairs.
  #[inline]
  pub fn iter(&self) -> impl Iterator<Item = (&[u8], &[u8])> {
    self.ab.iter().map(|el| (el.name_bytes, el.value_bytes))
  }

  /// The maximum allowed number of bytes.
  #[inline]
  pub fn max_bytes(&self) -> u32 {
    self.ab.max_bytes()
  }

  /// Removes the last element.
  #[inline]
  pub fn pop_back(&mut self) {
    self.ab.pop_back();
  }

  /// Removes the first element.
  #[inline]
  pub fn pop_front(&mut self) {
    self.ab.pop_front();
  }

  /// Pushes a new pair of `name` and `value` at the end of the internal buffer.
  ///
  /// If the sum of `name` and `value` is greater than the maximum number of bytes, then the first
  /// inserted entries will be deleted accordantly.
  #[inline]
  pub fn push(&mut self, name: &[u8], value: &[u8]) {
    self.ab.push((), name, value);
  }

  /// Removes all pairs referenced by the `names` parameter.
  #[inline]
  pub fn remove(&mut self, names: &[&[u8]]) {
    self.ab.remove(names);
  }

  /// If `max_bytes` is lesser than the current number of bytes, then the first inserted entries
  /// will be deleted accordantly.
  #[inline]
  pub fn set_max_bytes(&mut self, max_bytes: u32) {
    self.ab.set_max_bytes(max_bytes);
  }
}
