use crate::misc::simd_bytes_mut;

/// Mask Operation
///
/// Used to encode and decode payloads.
#[inline]
#[doc = _internal_doc!()]
pub(crate) fn mask_op(bytes: &mut [u8], mut mask: [u8; 4]) {
  _simd! {
    4 => {
      simd_bytes_mut(
        &mut mask,
        bytes,
        |local_mask, aligned| {
          for (array_elem, mask_elem) in aligned.iter_mut().zip(&*local_mask) {
            *array_elem ^= mask_elem;
          }
        }
        |local_mask, unaligned| fill_unaligned(unaligned, local_mask),
      );
    },
    16 => {
      simd_bytes_mut(
        &mut mask,
        bytes,
        |local_mask, aligned| {
          let [b0, b1, b2, b3] = *local_mask;
          let local_mask_bytes = [b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3];
          for (array_elem, mask_elem) in aligned.iter_mut().zip(&local_mask_bytes) {
            *array_elem ^= mask_elem;
          }
        },
        |local_mask, unaligned| fill_unaligned(unaligned, local_mask),
      );
    },
    32 => {
      simd_bytes_mut(
        &mut mask,
        bytes,
        |local_mask, aligned| {
          let [b0, b1, b2, b3] = *local_mask;
          let local_mask_bytes = [
            b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3,
            b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3, b0, b1, b2, b3
          ];
          for (array_elem, mask_elem) in aligned.iter_mut().zip(&local_mask_bytes) {
            *array_elem ^= mask_elem;
          }
        },
        |local_mask, unaligned| fill_unaligned(unaligned, local_mask),
      );
    },
    64 => {
      simd_bytes_mut(
        &mut mask,
        bytes,
        |local_mask, aligned| avx512(aligned, *local_mask),
        |local_mask, unaligned| fill_unaligned(unaligned, local_mask),
      );
    }
  }
}

// After several failed attempts it became clear that LLVM is allergic to AVX-512 in this
// particular workflow.
#[cfg(target_feature = "avx512f")]
#[inline(always)]
fn avx512(bytes: &mut [u8; 64], mask: [u8; 4]) {
  #[cfg(target_arch = "x86")]
  use std::arch::x86::{
    _mm512_load_si512, _mm512_set1_epi32, _mm512_store_si512, _mm512_xor_si512,
  };
  #[cfg(target_arch = "x86_64")]
  use std::arch::x86_64::{
    _mm512_load_si512, _mm512_set1_epi32, _mm512_store_si512, _mm512_xor_si512,
  };

  // SAFETY: Host is `x86` and `bytes` has 512 bits
  let local_bytes = unsafe { _mm512_load_si512(bytes.as_ptr().cast()) };
  // SAFETY: Host is `x86`
  let local_mask = unsafe { _mm512_set1_epi32(i32::from_ne_bytes(mask)) };
  // SAFETY: Host is `x86`
  let result = unsafe { _mm512_xor_si512(local_bytes, local_mask) };
  // SAFETY: Host is `x86`
  unsafe {
    _mm512_store_si512(bytes.as_mut_ptr().cast(), result);
  }
}

// For some reason LLVM is unrolling `bytes` using AVX2
#[inline(always)]
fn fill_unaligned(bytes: &mut [u8], mask: &mut [u8; 4]) {
  let (arrays, rem) = bytes.as_chunks_mut::<4>();
  let local_mask = u32::from_be_bytes(*mask);
  for array in arrays {
    let rslt = u32::from_be_bytes(*array) ^ local_mask;
    *array = rslt.to_be_bytes();
  }
  for (elem, mask_elem) in rem.iter_mut().zip(*mask) {
    *elem ^= mask_elem;
  }
  let shift = bytes.len() % 4;
  *mask = [
    mask.get(shift % 4).copied().unwrap_or_default(),
    mask.get(shift.wrapping_add(1) % 4).copied().unwrap_or_default(),
    mask.get(shift.wrapping_add(2) % 4).copied().unwrap_or_default(),
    mask.get(shift.wrapping_add(3) % 4).copied().unwrap_or_default(),
  ];
}

#[cfg(all(feature = "_bench", test))]
mod bench {
  use crate::bench::_data;

  #[bench]
  fn mask(b1: &mut test::Bencher) {
    let mut data = _data(1024 * 1024 * 8);
    b1.iter(|| crate::web_socket::mask_op::mask_op(&mut data, [3, 5, 7, 11]));
  }
}

#[cfg(kani)]
mod kani {
  use crate::collections::Vector;

  #[kani::proof]
  fn mask_op() {
    let mask = kani::any();
    let mut payload = Vector::from(kani::vec::any_vec::<u8, 128>());
    payload.fill(0);
    crate::web_socket::mask_op::mask_op(&mut payload, mask);
    let expected = Vector::from_iterator((0..payload.len()).map(|idx| mask[idx & 3])).unwrap();
    assert_eq!(payload, expected);
  }
}

#[cfg(test)]
mod tests {
  use crate::{collections::Vector, web_socket::mask_op::mask_op};

  #[test]
  fn length_variation_unmask() {
    for len in [0, 2, 3, 8, 16, 18, 31, 32, 40, 63, 100, 125, 256] {
      let mut payload = Vector::from_cloneable_elem(len, 0).unwrap();
      let mask = [1, 2, 3, 4];
      mask_op(&mut payload, mask);
      let expected = Vector::from_iterator((0..len).map(|idx| mask[idx & 3])).unwrap();
      assert_eq!(payload, expected);
    }
  }

  #[test]
  fn unmask_has_correct_output() {
    let mut payload = [0u8; 33];
    let mask = [1, 2, 3, 4];
    mask_op(&mut payload, mask);
    assert_eq!(
      &payload,
      &[
        1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2,
        3, 4, 1
      ]
    );
  }
}
