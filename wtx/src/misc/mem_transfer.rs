use core::{ops::Range, ptr};

/// Transfers sequential `iter` chunks delimited by indices to a region starting at `begin`.
///
/// ```ignore
/// // For example, a vector fragmented in 3 pieces where the last two digits of each piece are
/// // shifted to the left
///
///    |           |           |           |
/// A: |00|01|02|03|04|05|06|07|08|09|10|11|
///    |      << <<|           |           |
///
///    |           |           |           |
/// A: |02|03|02|03|04|05|06|07|08|09|10|11|
///    |^^ ^^      |      << <<|           |
///
///    |           |           |           |
/// A: |02|03|06|07|04|05|06|07|08|09|10|11|
///    |^^ ^^ ^^ ^^|           |      << <<|
///
///    |           |           |           |
/// A: |02|03|06|07|10|11|06|07|08|09|10|11|
///    |^^ ^^ ^^ ^^|^^ ^^      |           |
///
/// A: |02|03|06|07|10|11|
/// ```
#[inline]
pub(crate) fn _shift_bytes<T>(
  begin: usize,
  slice: &mut [T],
  iter: impl IntoIterator<Item = Range<usize>>,
) -> &mut [T]
where
  T: Copy,
{
  let mut new_len = begin;
  if new_len > slice.len() {
    return &mut [];
  }
  for Range { start, end } in iter {
    let Some((diff, local_new_len)) = end.checked_sub(start).and_then(|diff| {
      let local_new_len = new_len.checked_add(diff)?;
      if local_new_len > slice.len() {
        return None;
      }
      Some((diff, local_new_len))
    }) else {
      // SAFETY: Top-level and loop-level checks enforce bounds
      return unsafe { slice.get_unchecked_mut(..new_len) };
    };
    let ptr = slice.as_mut_ptr();
    // SAFETY: Loop-level check enforces bounds
    unsafe {
      ptr::copy(ptr.add(start), ptr.add(new_len), diff);
    }
    new_len = local_new_len;
  }
  // SAFETY: Top-level check enforces bounds
  unsafe { slice.get_unchecked_mut(..new_len) }
}

#[cfg(feature = "_proptest")]
#[cfg(test)]
mod proptest {
  use crate::misc::_shift_bytes;
  use alloc::vec::Vec;
  use core::ops::Range;

  #[test_strategy::proptest]
  fn shift_bytes(mut data: Vec<u8>, range: Range<u8>) {
    let mut begin: usize = range.start.into();
    let mut end: usize = range.end.into();
    let mut data_clone = data.clone();
    begin = begin.min(data.len());
    end = end.min(data.len());
    let rslt = _shift_bytes(0, &mut data, [begin..end]);
    data_clone.rotate_left(begin);
    data_clone.truncate(rslt.len());
    assert_eq!(rslt, &data_clone);
  }
}

#[cfg(test)]
mod test {
  use crate::misc::mem_transfer::_shift_bytes;

  #[test]
  fn _shift_bytes_has_correct_outputs() {
    let bytes = &mut [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    assert_eq!(_shift_bytes(2, bytes, [4..6, 8..10]), &mut [0, 1, 4, 5, 8, 9]);
    assert_eq!(bytes, &mut [0, 1, 4, 5, 8, 9, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
  }
}
