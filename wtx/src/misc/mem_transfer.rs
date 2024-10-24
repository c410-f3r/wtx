use core::{ops::Range, ptr};

/// Transfers sequential `iter` chunks delimited by indices to a region starting at `begin`.
///
/// ### Three delimited chunks shifted to the left
///
/// ```ignore
/// |00|01|02|03|04|05|06|07|08|09|10|11|
///        << <<
///
/// |02|03|02|03|04|05|06|07|08|09|10|11|
///  ^^ ^^             << <<
///
/// |02|03|06|07|04|05|06|07|08|09|10|11|
///  ^^ ^^ ^^ ^^                   << <<
///
/// |02|03|06|07|10|11|06|07|08|09|10|11|
///  ^^ ^^ ^^ ^^ ^^ ^^
///
/// |02|03|06|07|10|11|
/// ```
#[inline]
pub(crate) fn _shift_copyable_chunks<T>(
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
    let Some((diff @ 1..usize::MAX, local_new_len)) = end.checked_sub(start).and_then(|diff| {
      let local_new_len = new_len.checked_add(diff)?;
      if local_new_len > slice.len() {
        return None;
      }
      Some((diff, local_new_len))
    }) else {
      // SAFETY: top-level and loop-level checks enforce bounds
      return unsafe { slice.get_unchecked_mut(..new_len) };
    };
    let ptr = slice.as_mut_ptr();
    // SAFETY: loop-level check enforces bounds
    let src = unsafe { ptr.add(start) };
    // SAFETY: top-level check enforces bounds
    let dst = unsafe { ptr.add(new_len) };
    // SAFETY: loop-level check enforces a valid `diff`
    unsafe {
      ptr::copy(src, dst, diff);
    }
    new_len = local_new_len;
  }
  // SAFETY: Top-level check enforces bounds
  unsafe { slice.get_unchecked_mut(..new_len) }
}

#[cfg(all(feature = "_proptest", test))]
mod proptest {
  use crate::misc::{Vector, _shift_copyable_chunks};
  use core::ops::Range;

  #[test_strategy::proptest]
  fn shift_bytes(mut data: Vector<u8>, range: Range<u8>) {
    let mut begin: usize = range.start.into();
    let mut end: usize = range.end.into();
    let mut data_clone = data.clone();
    begin = begin.min(data.len());
    end = end.min(data.len());
    let rslt = _shift_copyable_chunks(0, &mut data, [begin..end]);
    data_clone.rotate_left(begin);
    data_clone.truncate(rslt.len());
    assert_eq!(rslt, data_clone.as_ref());
  }
}

#[cfg(test)]
mod test {
  use crate::misc::mem_transfer::_shift_copyable_chunks;

  #[test]
  fn _shift_bytes_has_correct_outputs() {
    let bytes = &mut [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    assert_eq!(_shift_copyable_chunks(2, bytes, [4..6, 8..10]), &mut [0, 1, 4, 5, 8, 9]);
    assert_eq!(bytes, &mut [0, 1, 4, 5, 8, 9, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
  }
}
