#[doc = _internal_doc!()]
#[inline]
pub(crate) fn unmask(bytes: &mut [u8], mask: [u8; 4]) {
  let mut mask_u32 = u32::from_ne_bytes(mask);
  // SAFETY: Changing a sequence of `u8` to `u32` should be fine
  let (prefix, words, suffix) = unsafe { bytes.align_to_mut::<u32>() };
  unmask_u8_slice(prefix, mask);
  let mut shift = u32::try_from(prefix.len() & 3).unwrap_or_default();
  if shift > 0 {
    shift = shift.wrapping_mul(8);
    if cfg!(target_endian = "big") {
      mask_u32 = mask_u32.rotate_left(shift);
    } else {
      mask_u32 = mask_u32.rotate_right(shift);
    }
  }
  unmask_u32_slice(words, mask_u32);
  unmask_u8_slice(suffix, mask_u32.to_ne_bytes());
}

#[expect(clippy::indexing_slicing, reason = "index will always be in-bounds")]
fn unmask_u8_slice(bytes: &mut [u8], mask: [u8; 4]) {
  for (idx, elem) in bytes.iter_mut().enumerate() {
    *elem ^= mask[idx & 3];
  }
}

fn unmask_u32_slice(bytes: &mut [u32], mask: u32) {
  _iter4_mut!(bytes, {}, |elem| {
    *elem ^= mask;
  });
}

#[cfg(feature = "_bench")]
#[cfg(test)]
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
  fn unmask(mut data: Vector<u8>, mask: [u8; 4]) {
    crate::web_socket::unmask(&mut data, mask);
  }
}

#[cfg(test)]
mod tests {
  use crate::web_socket::unmask::unmask;
  use alloc::{vec, vec::Vec};

  #[test]
  fn length_variation_unmask() {
    for len in &[0, 2, 3, 8, 16, 18, 31, 32, 40] {
      let mut payload = vec![0u8; *len];
      let mask = [1, 2, 3, 4];
      unmask(&mut payload, mask);
      let expected = (0..*len).map(|i| (i & 3) as u8 + 1).collect::<Vec<_>>();
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
