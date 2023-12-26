#![allow(
  // Indices point to valid memory
  clippy::unreachable
)]

use alloc::vec::Vec;

/// Helper that manages the copy of initialized bytes.
#[derive(Debug)]
pub struct FilledBufferWriter<'vec> {
  _curr_idx: usize,
  _initial_idx: usize,
  _vec: &'vec mut Vec<u8>,
}

impl<'vec> FilledBufferWriter<'vec> {
  pub(crate) fn new(start: usize, vec: &'vec mut Vec<u8>) -> Self {
    Self { _curr_idx: start, _initial_idx: start, _vec: vec }
  }

  pub(crate) fn _available_bytes_mut(&mut self) -> &mut [u8] {
    if let Some(elem) = self._vec.get_mut(self._curr_idx..) {
      elem
    } else {
      unreachable!()
    }
  }

  pub(crate) fn _curr_bytes(&self) -> &[u8] {
    if let Some(elem) = self._vec.get(self._initial_idx..self._curr_idx) {
      elem
    } else {
      unreachable!()
    }
  }

  pub(crate) fn _curr_bytes_mut(&mut self) -> &mut [u8] {
    if let Some(elem) = self._vec.get_mut(self._initial_idx..self._curr_idx) {
      elem
    } else {
      unreachable!()
    }
  }

  pub(crate) fn _len(&self) -> usize {
    self._curr_idx.wrapping_sub(self._initial_idx)
  }

  pub(crate) fn _extend_from_byte(&mut self, byte: u8) {
    let new_start = self._curr_idx.wrapping_add(1);
    self._expand_buffer(new_start);
    let Some(vec_byte) = self._vec.get_mut(self._curr_idx) else {
      unreachable!();
    };
    *vec_byte = byte;
    self._curr_idx = new_start;
  }

  pub(crate) fn _extend_from_slice(&mut self, slice: &[u8]) {
    self._extend_from_slice_generic(slice, []);
  }

  pub(crate) fn _extend_from_slices(&mut self, slices: &[&[u8]]) {
    for slice in slices {
      self._extend_from_slice(slice);
    }
  }

  /// The `c` suffix means that `slice` is copied as a C string.
  pub(crate) fn _extend_from_slice_c(&mut self, slice: &[u8]) {
    self._extend_from_slice_generic(slice, [0]);
  }

  /// The `each_c` suffix means that each slice is copied as a C string.
  pub(crate) fn _extend_from_slices_each_c(&mut self, slices: &[&[u8]]) {
    for slice in slices {
      self._extend_from_slice_c(slice);
    }
  }

  /// The `rn` suffix means that `slice` is copied with a final `\r\n` new line.
  pub(crate) fn _extend_from_slice_rn(&mut self, slice: &[u8]) {
    self._extend_from_slice_generic(slice, *b"\r\n");
  }

  /// The `group_rn` suffix means that only the last slice is copied with a final `\r\n` new line.
  pub(crate) fn _extend_from_slices_group_rn(&mut self, slices: &[&[u8]]) {
    if let [rest @ .., last] = slices {
      for slice in rest {
        self._extend_from_slice(slice);
      }
      self._extend_from_slice_rn(last);
    }
  }

  pub(crate) fn _shift_idx(&mut self, n: usize) {
    let new_len = self._curr_idx.wrapping_add(n);
    self._expand_buffer(new_len);
    self._curr_idx = new_len;
  }

  fn _expand_buffer(&mut self, new_len: usize) {
    if new_len > self._vec.len() {
      self._vec.resize(new_len, 0);
    }
  }

  fn _extend_from_slice_generic<const N: usize>(&mut self, slice: &[u8], suffix: [u8; N]) {
    let until_slice = self._curr_idx.wrapping_add(slice.len());
    let until_suffix = until_slice.wrapping_add(N);
    self._expand_buffer(until_suffix);
    if let Some(vec_slice) = self._vec.get_mut(self._curr_idx..until_slice) {
      vec_slice.copy_from_slice(slice);
    } else {
      unreachable!();
    }
    if let Some(vec_slice) = self._vec.get_mut(until_slice..until_suffix) {
      vec_slice.copy_from_slice(&suffix);
    } else {
      unreachable!();
    }
    self._curr_idx = until_suffix;
  }
}

#[cfg(test)]
mod tests {
  #[cfg(feature = "_bench")]
  #[bench]
  fn extend_from_slice(b: &mut test::Bencher) {
    use alloc::{vec, vec::Vec};
    let array: [u8; 64] = core::array::from_fn(|idx| {
      let n = idx % 255;
      n.try_into().unwrap_or(u8::MAX)
    });
    let mut vec = vec![0; 128];
    let mut fbw = crate::misc::FilledBufferWriter::new(32, &mut vec);
    b.iter(|| {
      fbw._extend_from_slice(&array);
    });
  }
}
