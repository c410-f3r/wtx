#[doc = _internal_doc!()]
pub(crate) fn unmask(bytes: &mut [u8], mask: [u8; 4]) {
  let [a, b, c, d] = mask;
  _simd_slice!(
    (),
    (bytes,),
    (_, (local_bytes,)) => {
      _do_unmask(&[a, b, c, d, a, b, c, d], local_bytes);
    },
    (_, (local_bytes,)) => {
      _do_unmask(&[a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d], local_bytes);
    },
    (_, (local_bytes,)) => {
      _do_unmask(
        &[
          a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b,
          c, d,
        ],
        local_bytes,
      );
    },
    (_, (local_bytes,)) => {
      _do_unmask(
        &[
          a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b,
          c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d,
          a, b, c, d,
        ],
        local_bytes,
      );
    },
    (_, (rest,)) => {
      for (elem, mask_elem) in rest.iter_mut().zip(mask.into_iter().cycle()) {
        *elem ^= mask_elem;
      }
    },
  );
}

fn _do_unmask<const N: usize>(mask: &[u8], slice: &mut [[u8; N]]) {
  for array in slice {
    for (array_elem, mask_elem) in array.iter_mut().zip(mask) {
      *array_elem ^= mask_elem;
    }
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
  use crate::collection::Vector;

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
  use crate::{collection::Vector, web_socket::unmask::unmask};

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
