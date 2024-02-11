#![allow(
  // See safety comments
  unsafe_code
)]

//! On a fragmented vector of bytes, performs several experiments to see which is faster:
//!
//! 1. Transfer interval fragments to another vector (memcpy).
//!
//! ```ignore
//! // A vector fragmented in 3 pieces where the last two digits of each piece is copied
//! // to another vector.
//!
//!    |           |           |           |
//! A: |00|01|02|03|04|05|06|07|08|09|10|11|
//!    |     |_____|     |_____|     |_____|        
//!    |        |  |        |  |        |  |
//!    |       \|/ |       \|/ |       \|/ |
//!
//! B: |02|03|06|07|10|11|
//! ```
//!
//! 2. In-place shifts of interval fragments (memmove).
//!
//! ```ignore
//! // A vector fragmented in 3 pieces where the last two digits of each piece are
//! // shifted to the left
//!
//!    |           |           |           |
//! A: |00|01|02|03|04|05|06|07|08|09|10|11|
//!    |      << <<|           |           |
//!
//!    |           |           |           |
//! A: |02|03|02|03|04|05|06|07|08|09|10|11|
//!    |^^ ^^      |      << <<|           |
//!
//!    |           |           |           |
//! A: |02|03|06|07|04|05|06|07|08|09|10|11|
//!    |^^ ^^ ^^ ^^|           |      << <<|
//!
//!    |           |           |           |
//! A: |02|03|06|07|10|11|06|07|08|09|10|11|
//!    |^^ ^^ ^^ ^^|^^ ^^      |           |
//!
//! A: |02|03|06|07|10|11|
//! ```

use crate::misc::{_unlikely_cb, _unlikely_elem};
use core::ptr;

#[cfg(test)]
pub(crate) fn _copy_bytes<'to>(
  from: &[u8],
  to: &'to mut [u8],
  start: usize,
  iter: impl IntoIterator<Item = (usize, usize)>,
) -> &'to mut [u8] {
  if start >= from.len() || start >= to.len() {
    return _unlikely_elem(to);
  }
  let mut new_len = start;
  unsafe {
    ptr::copy_nonoverlapping(from.as_ptr(), to.as_mut_ptr(), start);
  }
  for (begin, end) in iter {
    let len = end.wrapping_sub(begin);
    let to_end = new_len.wrapping_add(len);
    let from_is_not_valid = from.get(begin..end).is_none();
    let to_is_not_valid = to.get(new_len..to_end).is_none();
    if from_is_not_valid || to_is_not_valid {
      return _unlikely_cb(|| unsafe { to.get_unchecked_mut(..new_len) });
    }
    unsafe {
      ptr::copy_nonoverlapping(from.as_ptr().add(begin), to.as_mut_ptr().add(new_len), len);
    }
    new_len = to_end;
  }
  unsafe { to.get_unchecked_mut(..new_len) }
}

pub(crate) fn _shift_bytes(
  slice: &mut [u8],
  start: usize,
  iter: impl IntoIterator<Item = (usize, usize)>,
) -> &mut [u8] {
  if start >= slice.len() {
    return _unlikely_elem(slice);
  }
  let mut new_len = start;
  for (begin, end) in iter {
    if slice.get(begin..end).is_none() {
      // SAFETY: Top-level check enforces bounds
      return _unlikely_cb(|| unsafe { slice.get_unchecked_mut(..new_len) });
    }
    let len = end.wrapping_sub(begin);
    // SAFETY: Loop-level check enforces bounds
    unsafe {
      ptr::copy(slice.as_ptr().add(begin), slice.as_mut_ptr().add(new_len), len);
    }
    new_len = new_len.wrapping_add(len);
  }
  // SAFETY: Top-level check enforces bounds
  unsafe { slice.get_unchecked_mut(..new_len) }
}

#[cfg(feature = "_bench")]
#[cfg(test)]
mod bench {
  use crate::misc::mem_transfer::{_copy_bytes, _shift_bytes};
  use alloc::{vec, vec::Vec};
  use core::hint::black_box;

  const SHIFT_LEN: usize = 1000;
  const SLICES_LEN: usize = 10000000;
  const SLICES_NUM: usize = 1000;

  #[bench]
  fn copy_bytes(b: &mut test::Bencher) {
    let from: Vec<u8> = crate::bench::_data(SLICES_LEN * SLICES_NUM);
    let mut to = vec![0; SLICES_LEN * SLICES_NUM];
    b.iter(|| {
      black_box({
        let iter = (0..SLICES_NUM).skip(1).scan(SLICES_LEN, |begin, _| {
          let end = begin.wrapping_add(SHIFT_LEN);
          let rslt = (*begin, end);
          *begin = begin.wrapping_add(SLICES_LEN);
          Some(rslt)
        });
        assert_eq!(_copy_bytes(&from, &mut to, 0, iter).len(), SHIFT_LEN * (SLICES_NUM - 1));
      })
    });
  }

  #[bench]
  fn shift_bytes(b: &mut test::Bencher) {
    let mut vector = crate::bench::_data(SLICES_LEN * SLICES_NUM);
    b.iter(|| {
      black_box({
        let iter = (0..SLICES_NUM).skip(1).scan(SLICES_LEN, |begin, _| {
          let end = begin.wrapping_add(SHIFT_LEN);
          let rslt = (*begin, end);
          *begin = begin.wrapping_add(SLICES_LEN);
          Some(rslt)
        });
        assert_eq!(_shift_bytes(&mut vector, 0, iter).len(), SHIFT_LEN * (SLICES_NUM - 1));
      })
    });
  }
}

#[cfg(feature = "_proptest")]
#[cfg(test)]
mod proptest {
  use crate::misc::{_shift_bytes, mem_transfer::_copy_bytes};
  use core::ops::Range;

  #[test_strategy::proptest]
  fn copy_bytes(from: Vec<u8>, range: Range<u8>) {
    let mut begin: usize = range.start.into();
    let mut end: usize = range.end.into();
    let mut from_clone = from.clone();
    let mut to = vec![0; from.len()];
    begin = begin.min(from.len());
    end = end.min(from.len());
    let rslt = _copy_bytes(&from, &mut to, 0, [(begin, end)]);
    from_clone.rotate_left(begin);
    from_clone.truncate(rslt.len());
    assert_eq!(rslt, &from_clone);
  }

  #[test_strategy::proptest]
  fn shift_bytes(mut data: Vec<u8>, range: Range<u8>) {
    let mut begin: usize = range.start.into();
    let mut end: usize = range.end.into();
    let mut data_clone = data.clone();
    begin = begin.min(data.len());
    end = end.min(data.len());
    let rslt = _shift_bytes(&mut data, 0, [(begin, end)]);
    data_clone.rotate_left(begin);
    data_clone.truncate(rslt.len());
    assert_eq!(rslt, &data_clone);
  }
}

#[cfg(test)]
mod test {
  use crate::misc::mem_transfer::{_copy_bytes, _shift_bytes};

  #[test]
  fn _copy_bytes_has_correct_outputs() {
    let from = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let to = &mut [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    assert_eq!(_copy_bytes(from, to, 2, [(4, 6), (8, 10)]), &mut [0, 1, 4, 5, 8, 9]);
    assert_eq!(to, &mut [0, 1, 4, 5, 8, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
  }

  #[test]
  fn _shift_bytes_has_correct_outputs() {
    let bytes = &mut [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    assert_eq!(_shift_bytes(bytes, 2, [(4, 6), (8, 10)]), &mut [0, 1, 4, 5, 8, 9]);
    assert_eq!(bytes, &mut [0, 1, 4, 5, 8, 9, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
  }
}
