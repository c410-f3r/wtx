use crate::_SIMD_LEN;

#[doc = _internal_doc!()]
pub(crate) fn unmask(bytes: &mut [u8], [b0, b1, b2, b3]: [u8; 4]) {
  let (arrays, rem) = bytes.as_chunks_mut::<{ _SIMD_LEN }>();

  let local_mask = _simd! {
    4 => [b0, b1, b2, b3],
    16 => [b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3],
    32 => [
      b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3,
      b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3
    ],
    64 => [
      b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3,
      b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3,
      b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3,
      b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3,
    ]
  };

  for array in arrays {
    for (array_elem, mask_elem) in array.iter_mut().zip(&local_mask) {
      *array_elem ^= mask_elem;
    }
  }

  for (elem, mask_elem) in rem.iter_mut().zip(&local_mask) {
    *elem ^= mask_elem;
  }
}

#[cfg(all(feature = "_bench", test))]
mod bench {
  use crate::bench::_data;

  #[bench]
  fn unmask(b1: &mut test::Bencher) {
    let mut data = _data(1024 * 1024 * 8);
    b1.iter(|| crate::web_socket::unmask::unmask(&mut data, [3, 5, 7, 11]));
  }
}

#[cfg(kani)]
mod kani {
  use crate::collection::Vector;

  #[kani::proof]
  fn unmask() {
    let mask = kani::any();
    let mut payload = Vector::from(kani::vec::any_vec::<u8, 128>());
    payload.fill(0);
    crate::web_socket::unmask::unmask(&mut payload, mask);
    let expected = Vector::from_iterator((0..payload.len()).map(|idx| mask[idx & 3])).unwrap();
    assert_eq!(payload, expected);
  }
}

#[cfg(test)]
mod tests {
  use crate::{collection::Vector, web_socket::unmask::unmask};

  #[test]
  fn length_variation_unmask() {
    for len in [0, 2, 3, 8, 16, 18, 31, 32, 40, 63, 100, 125, 256] {
      let mut payload = Vector::from_cloneable_elem(len, 0).unwrap();
      let mask = [1, 2, 3, 4];
      unmask(&mut payload, mask);
      let expected = Vector::from_iterator((0..len).map(|idx| mask[idx & 3])).unwrap();
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
