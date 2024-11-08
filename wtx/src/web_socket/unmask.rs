#[doc = _internal_doc!()]
#[inline]
pub(crate) fn unmask(bytes: &mut [u8], mut mask: [u8; 4]) {
  let unmask_chunks_slice = _simd!(
    fallback => _unmask_chunks_slice_fallback,
    128 => _unmask_chunks_slice_128,
    256 => _unmask_chunks_slice_256,
    512 => _unmask_chunks_slice_512
  );
  // SAFETY: Changing a sequence of `u8` should be fine
  let (prefix, chunks, suffix) = unsafe { bytes.align_to_mut() };
  unmask_u8_slice(prefix, mask, 0);
  mask.rotate_left(prefix.len() % 4);
  unmask_chunks_slice(chunks, mask);
  unmask_u8_slice(suffix, mask, 0);
}

#[inline]
fn _unmask_chunks_slice_512(slice: &mut [[u8; 64]], [a, b, c, d]: [u8; 4]) {
  let mask = [
    a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d,
    a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d,
  ];
  for array in slice {
    for (array_elem, mask_elem) in array.iter_mut().zip(mask) {
      *array_elem ^= mask_elem;
    }
  }
}

#[inline]
fn _unmask_chunks_slice_256(slice: &mut [[u8; 32]], [a, b, c, d]: [u8; 4]) {
  let mask = [
    a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d,
  ];
  for array in slice {
    for (array_elem, mask_elem) in array.iter_mut().zip(mask) {
      *array_elem ^= mask_elem;
    }
  }
}

#[inline]
fn _unmask_chunks_slice_128(slice: &mut [[u8; 16]], [a, b, c, d]: [u8; 4]) {
  let mask = [a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d];
  for array in slice {
    for (array_elem, mask_elem) in array.iter_mut().zip(mask) {
      *array_elem ^= mask_elem;
    }
  }
}

#[inline]
fn _unmask_chunks_slice_fallback(bytes: &mut [u8], mask: [u8; 4]) {
  unmask_u8_slice(bytes, mask, 0);
}

#[expect(clippy::indexing_slicing, reason = "index will always be in-bounds")]
#[inline]
fn unmask_u8_slice(bytes: &mut [u8], mask: [u8; 4], shift: usize) {
  for (idx, elem) in bytes.iter_mut().enumerate() {
    *elem ^= mask[idx.wrapping_add(shift) & 3];
  }
}

#[cfg(all(feature = "_bench", test))]
mod bench {
  use crate::bench::_data;

  #[bench]
  fn unmask(b: &mut test::Bencher) {
    let mut data = _data(1024 * 1024 * 8);
    b.iter(|| crate::web_socket::unmask::unmask(&mut data, [3, 5, 7, 11]));
  }
}

#[cfg(kani)]
mod kani {
  use crate::misc::Vector;

  #[kani::proof]
  fn unmask() {
    let mask = kani::any();
    let mut payload = Vector::from(kani::vec::any_vec::<u8, 128>());
    payload.fill(0);
    crate::web_socket::unmask::unmask(&mut payload, mask);
    let expected = Vector::from_iter((0..payload.len()).map(|idx| mask[idx & 3])).unwrap();
    assert_eq!(payload, expected);
  }
}

#[cfg(test)]
mod tests {
  use crate::{misc::Vector, web_socket::unmask::unmask};

  #[test]
  fn length_variation_unmask() {
    for len in [0, 2, 3, 8, 16, 18, 31, 32, 40, 63, 100, 125, 256] {
      let mut payload = Vector::from_cloneable_elem(len, 0).unwrap();
      let mask = [1, 2, 3, 4];
      unmask(&mut payload, mask);
      let expected = Vector::from_iter((0..len).map(|idx| mask[idx & 3])).unwrap();
      assert_eq!(payload, expected);
    }
  }

  #[test]
  fn unmask_has_correct_output() {
    let mut payload = [0u8; 33];
    let mask = [1, 2, 3, 4];
    unmask(&mut payload, mask);
    assert_eq!(
      &payload,
      &[
        1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2,
        3, 4, 1
      ]
    );
  }
}
