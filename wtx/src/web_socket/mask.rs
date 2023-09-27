/// Unmasks a sequence of bytes using the given 4-byte `mask`.
#[inline]
pub fn unmask(bytes: &mut [u8], mask: [u8; 4]) {
  let mut mask_u32 = u32::from_ne_bytes(mask);
  #[allow(unsafe_code)]
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

#[allow(
  // Index will always by in-bounds.
  clippy::indexing_slicing
)]
fn unmask_u8_slice(bytes: &mut [u8], mask: [u8; 4]) {
  for (idx, elem) in bytes.iter_mut().enumerate() {
    *elem ^= mask[idx & 3];
  }
}

fn unmask_u32_slice(bytes: &mut [u32], mask: u32) {
  macro_rules! loop_chunks {
    ($bytes:expr, $mask:expr, $($elem:ident),* $(,)?) => {{
      let mut iter = $bytes.array_chunks_mut::<{ 0 $( + { let $elem = 1; $elem })* }>();
      for [$($elem,)*] in iter.by_ref() {
        $( *$elem ^= $mask; )*
      }
      iter
    }};
  }
  loop_chunks!(bytes, mask, _1, _2, _3, _4)
    .into_remainder()
    .iter_mut()
    .for_each(|elem| *elem ^= mask);
}

#[cfg(test)]
mod tests {
  use crate::web_socket::mask::unmask;
  use alloc::{vec, vec::Vec};

  #[test]
  fn test_unmask() {
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
}
