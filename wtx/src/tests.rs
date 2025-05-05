use crate::{
  misc::UriString,
  rng::{Rng, Xorshift64, simple_seed},
  sync::{AtomicU32, Ordering},
};

pub(crate) fn _uri() -> UriString {
  static PORT: AtomicU32 = AtomicU32::new(7000);
  let uri = alloc::format!("http://127.0.0.1:{}", PORT.fetch_add(1, Ordering::Relaxed));
  UriString::new(uri)
}

pub(crate) fn _32_bytes_seed() -> [u8; 32] {
  let seed = simple_seed();
  let mut rng = Xorshift64::from(seed);
  let [a0, b0, c0, d0, e0, f0, g0, h0, i0, j0, k0, l0, m0, n0, o0, p0] = rng.u8_16();
  let [a1, b1, c1, d1, e1, f1, g1, h1, i1, j1, k1, l1, m1, n1, o1, p1] = rng.u8_16();
  [
    a0, b0, c0, d0, e0, f0, g0, h0, i0, j0, k0, l0, m0, n0, o0, p0, a1, b1, c1, d1, e1, f1, g1, h1,
    i1, j1, k1, l1, m1, n1, o1, p1,
  ]
}
