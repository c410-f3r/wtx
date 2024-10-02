#[doc = _internal_doc!()]
#[inline]
pub(crate) fn unmask(bytes: &mut [u8], mut mask: [u8; 4]) {
  #[cfg(target_feature = "avx512f")]
  let (is_128, unmask_chunks_slice) = (false, _unmask_chunks_slice_512);

  #[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
  let (is_128, unmask_chunks_slice) = (false, _unmask_chunks_slice_256);

  #[cfg(all(
    target_feature = "neon",
    not(any(target_feature = "avx2", target_feature = "avx512f"))
  ))]
  let (is_128, unmask_chunks_slice) = (true, _unmask_chunks_slice_128);

  #[cfg(all(
    target_feature = "sse2",
    not(any(target_feature = "avx2", target_feature = "avx512f", target_feature = "neon"))
  ))]
  let (is_128, unmask_chunks_slice) = (true, _unmask_chunks_slice_128);

  #[cfg(not(any(
    target_feature = "avx2",
    target_feature = "avx512f",
    target_feature = "neon",
    target_feature = "sse2"
  )))]
  let (is_128, unmask_chunks_slice) = (false, _unmask_chunks_slice_fallback);

  // SAFETY: Changing a sequence of `u8` should be fine
  let (prefix, chunks, suffix) = unsafe { bytes.align_to_mut() };
  unmask_u8_slice(prefix, mask, 0);
  mask.rotate_left(prefix.len() % 4);
  unmask_chunks_slice(chunks, mask);
  unmask_u8_slice(suffix, mask, if is_128 { (chunks.len() % 2).wrapping_mul(2) } else { 0 });
}

#[inline]
fn _unmask_chunks_slice_512(bytes: &mut [u64], [a, b, c, d]: [u8; 4]) {
  let mask = u64::from_be_bytes([d, c, b, a, d, c, b, a]);
  for elem in bytes {
    *elem ^= mask;
  }
}

#[inline]
fn _unmask_chunks_slice_256(bytes: &mut [u32], [a, b, c, d]: [u8; 4]) {
  let mask = u32::from_be_bytes([d, c, b, a]);
  for elem in bytes {
    *elem ^= mask;
  }
}

#[expect(clippy::indexing_slicing, reason = "index will always be in-bounds")]
#[inline]
fn _unmask_chunks_slice_128(bytes: &mut [u16], [a, b, c, d]: [u8; 4]) {
  let mask = [u16::from_be_bytes([b, a]), u16::from_be_bytes([d, c])];
  for (idx, elem) in bytes.iter_mut().enumerate() {
    *elem ^= mask[idx & 1];
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
  use crate::{bench::_data, web_socket::unmask};

  #[bench]
  fn bench_unmask(b: &mut test::Bencher) {
    let mut data = _data(1024 * 1024 * 8);
    b.iter(|| unmask(&mut data, [3, 5, 7, 11]));
  }
}

#[cfg(test)]
#[cfg(feature = "_proptest")]
mod proptest {
  use crate::misc::Vector;

  #[test_strategy::proptest]
  fn unmask(mut payload: Vector<u8>, mask: [u8; 4]) {
    payload.fill(0);
    crate::web_socket::unmask(&mut payload, mask);
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
