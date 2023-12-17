use alloc::vec::Vec;

/// List of pairs sent and received on every request.
#[derive(Debug, Default)]
pub struct WtxHeader {
  buffer: Vec<u8>,
  headers_len: usize,
  indcs: Vec<(bool, usize, usize)>,
}

impl WtxHeader {
  /// Clears the internal buffer "erasing" all previously inserted elements.
  #[inline]
  pub fn clear(&mut self) {
    self.buffer.clear();
    self.indcs.clear();
  }

  /// Returns the header value of the **first** corresponding header `name` key, if any.
  #[inline]
  pub fn get(&self, name: &[u8]) -> Option<&[u8]> {
    self.iter().find_map(|[key, value]| (name == key).then_some(value))
  }

  /// The amount of bytes used by all of the headers
  #[inline]
  pub fn headers_len(&self) -> usize {
    self.headers_len
  }

  /// Retrieves all stored pairs.
  #[inline]
  pub fn iter(&self) -> impl Iterator<Item = [&[u8]; 2]> {
    self.indcs.iter().copied().scan(0, |idx_tracker, (activated, key_idx, value_idx)| {
      if !activated {
        return None;
      }
      let key_str = self.buffer.get(*idx_tracker..key_idx)?;
      let value_str = self.buffer.get(key_idx..value_idx)?;
      *idx_tracker = value_idx;
      Some([key_str, value_str])
    })
  }

  /// Pushes a new pair of `key` and `value` at the end of the internal buffer.
  #[inline]
  pub fn push_bytes(&mut self, key: &[u8], value: &[u8]) {
    self.buffer.extend(key);
    let key_idx = self.buffer.len();
    self.buffer.extend(value);
    let value_idx = self.buffer.len();
    self.indcs.push((true, key_idx, value_idx));
    self.headers_len = self.headers_len.wrapping_add(key.len());
    self.headers_len = self.headers_len.wrapping_add(value.len());
  }

  /// Removes all pairs referenced by the `keys` names.
  #[inline]
  pub fn remove(&mut self, keys: &[&[u8]]) {
    let mut keys_start = 0;
    let mut idx_tracker = 0;
    for (activated, key_idx, value_idx) in &mut self.indcs {
      if !*activated {
        continue;
      }
      let tuple = (self.buffer.get(idx_tracker..*key_idx), keys.get(keys_start..));
      let (Some(key_str), Some(slice)) = tuple else {
        break;
      };
      if slice.contains(&key_str) {
        *activated = false;
        keys_start = keys_start.wrapping_add(1);
        self.headers_len = self.headers_len.wrapping_sub(key_str.len());
        self.headers_len = self.headers_len.wrapping_sub(value_idx.wrapping_sub(*key_idx));
      }
      idx_tracker = *value_idx;
    }
  }
}
