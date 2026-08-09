use crate::_SIMD_LEN;

/// Processes a sequence of bytes with the most suitable simd length according to the current host.
///
/// In Alder Lake the cache line size is 64 bytes, which matches the length of AVX-512 registers.
/// Because of that it is important to perform aligned reads to avoid having to fetch 2 cache lines
/// instead of just 1. However, intra cache line operations don't seem to suffer much from
/// unaligned.
#[inline(always)]
pub fn simd_bytes<A>(
  aux: &mut A,
  bytes: &[u8],
  mut aligned: impl FnMut(&mut A, &[u8; _SIMD_LEN]),
  mut unaligned: impl FnMut(&mut A, &[u8]),
) {
  // SAFETY: From bytes to bytes, the method is just logically separating chunks.
  let (prefix, data, suffix) = unsafe { bytes.align_to::<Simd>() };
  unaligned(aux, prefix);
  for elem in data {
    aligned(aux, &elem.0);
  }
  unaligned(aux, suffix);
}

/// Mutable version of [`simd_bytes`].
#[inline(always)]
pub fn simd_bytes_mut<A>(
  aux: &mut A,
  bytes: &mut [u8],
  mut aligned: impl FnMut(&mut A, &mut [u8; _SIMD_LEN]),
  mut unaligned: impl FnMut(&mut A, &mut [u8]),
) {
  // SAFETY: From bytes to bytes, the method is just logically separating chunks.
  let (prefix, data, suffix) = unsafe { bytes.align_to_mut::<Simd>() };
  unaligned(aux, prefix);
  for elem in data {
    aligned(aux, &mut elem.0);
  }
  unaligned(aux, suffix);
}

_simd! {
  4 => {
    #[repr(align(4))]
    struct Simd([u8; 4]);
  },
  16 => {
    #[repr(align(16))]
    struct Simd([u8; 16]);
  },
  32 => {
    #[repr(align(32))]
    struct Simd([u8; 32]);
  },
  64 => {
    #[repr(align(64))]
    struct Simd([u8; 64]);
  }
}
