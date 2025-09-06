#![allow(clippy::wrong_self_convention, reason = "`to_*` is allowed for self references")]

// 1 iteration = 2 rounds = 8 quarter rounds

use crate::rng::{CryptoRng, Rng, SeedableRng};
use core::fmt::Debug;

const ITERATIONS: u8 = 10;
// Each word has 32 bits or 4 bytes
const WORDS: usize = 16;

/// `ChaCha` block function with 20 rounds and a nonce of 12 bytes as specified in
/// <https://datatracker.ietf.org/doc/html/rfc7539>.
///
/// This structure is `Copy` to allow usage with `AtomicCell` in concurrent scenarios. You should
/// probably use other implementations if performance or auditability is a concern.
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct ChaCha20 {
  block: Block,
  idx: u8,
  output: [u32; WORDS],
}

impl ChaCha20 {
  /// Creates a new instance where you are responsible for providing parameters. Ideally, `key`
  /// should have a high entropy.
  #[inline]
  pub const fn new(key: [u8; 32]) -> ChaCha20 {
    ChaCha20 { block: Block::new(key, [0; 12]), idx: 16, output: [0; WORDS] }
  }

  fn increment_counter(&mut self) {
    let fun = |num: &mut u32| {
      let (rslt, overflow) = num.overflowing_add(1);
      *num = rslt;
      !overflow
    };
    if fun(&mut self.block.block_counter) {
      return;
    }
    if fun(self.block.first_nonce_mut()) {
      return;
    }
    if fun(self.block.second_nonce_mut()) {
      return;
    }
    let _ = fun(self.block.third_nonce_mut());
  }
}

impl CryptoRng for ChaCha20 {}

impl Debug for ChaCha20 {
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("ChaCha20").finish()
  }
}

impl Rng for ChaCha20 {
  #[inline]
  fn u8(&mut self) -> u8 {
    self.u8_4()[0]
  }

  #[inline]
  fn u8_4(&mut self) -> [u8; 4] {
    if usize::from(self.idx) == WORDS {
      let lock_block = block_function::<true>(&self.block);
      self.idx = 0;
      self.output = lock_block.to_words();
      self.increment_counter();
    }
    let rslt = self.output.get(usize::from(self.idx) % WORDS).copied().unwrap_or_default();
    self.idx = self.idx.wrapping_add(1);
    rslt.to_le_bytes()
  }

  #[inline]
  fn u8_8(&mut self) -> [u8; 8] {
    let [a, b, c, d] = self.u8_4();
    let [e, f, g, h] = self.u8_4();
    [a, b, c, d, e, f, g, h]
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    let [a, b, c, d] = self.u8_4();
    let [e, f, g, h] = self.u8_4();
    let [i, j, k, l] = self.u8_4();
    let [m, n, o, p] = self.u8_4();
    [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p]
  }
}

impl SeedableRng for ChaCha20 {
  type Seed = [u8; 32];

  #[inline]
  fn from_seed(seed: Self::Seed) -> crate::Result<Self> {
    Ok(Self::new(seed))
  }
}

#[cfg_attr(test, derive(Debug))]
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct Block {
  constants: [u32; 4],
  keys: [u32; 8],
  block_counter: u32,
  nonces: [u32; 3],
}

impl Block {
  fn from_rows(a: Row, b: Row, c: Row, d: Row) -> Self {
    let Row(a0, a1, a2, a3) = a;
    let Row(b0, b1, b2, b3) = b;
    let Row(c0, c1, c2, c3) = c;
    let Row(d0, d1, d2, d3) = d;
    Self {
      constants: [a0, a1, a2, a3],
      keys: [b0, b1, b2, b3, c0, c1, c2, c3],
      block_counter: d0,
      nonces: [d1, d2, d3],
    }
  }

  #[cfg(test)]
  fn from_words(words: [u32; WORDS]) -> Self {
    let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = words;
    Self {
      constants: [a, b, c, d],
      keys: [e, f, g, h, i, j, k, l],
      block_counter: m,
      nonces: [n, o, p],
    }
  }

  const fn new(key: [u8; 32], nonce: [u8; 12]) -> Self {
    #[rustfmt::skip]
    let [
      k0, k1, k2, k3, k4, k5, k6, k7, k8, k9, k10, k11, k12, k13, k14, k15,
      k16, k17, k18, k19, k20, k21, k22, k23, k24, k25, k26, k27, k28, k29, k30, k31,
    ] = key;
    let [n0, n1, n2, n3, n4, n5, n6, n7, n8, n9, n10, n11] = nonce;
    Self {
      constants: [0x6170_7865, 0x3320_646E, 0x7962_2D32, 0x6B2_06574],
      keys: [
        u32::from_le_bytes([k0, k1, k2, k3]),
        u32::from_le_bytes([k4, k5, k6, k7]),
        u32::from_le_bytes([k8, k9, k10, k11]),
        u32::from_le_bytes([k12, k13, k14, k15]),
        u32::from_le_bytes([k16, k17, k18, k19]),
        u32::from_le_bytes([k20, k21, k22, k23]),
        u32::from_le_bytes([k24, k25, k26, k27]),
        u32::from_le_bytes([k28, k29, k30, k31]),
      ],
      block_counter: 0,
      nonces: [
        u32::from_le_bytes([n0, n1, n2, n3]),
        u32::from_le_bytes([n4, n5, n6, n7]),
        u32::from_le_bytes([n8, n9, n10, n11]),
      ],
    }
  }

  fn first_nonce_mut(&mut self) -> &mut u32 {
    &mut self.nonces[0]
  }

  fn second_nonce_mut(&mut self) -> &mut u32 {
    &mut self.nonces[1]
  }

  fn third_nonce_mut(&mut self) -> &mut u32 {
    &mut self.nonces[2]
  }

  fn to_rows(&self) -> (Row, Row, Row, Row) {
    let [c0, c1, c2, c3] = self.constants;
    let [k0, k1, k2, k3, k4, k5, k6, k7] = self.keys;
    let [n0, n1, n2] = self.nonces;
    (
      Row(c0, c1, c2, c3),
      Row(k0, k1, k2, k3),
      Row(k4, k5, k6, k7),
      Row(self.block_counter, n0, n1, n2),
    )
  }

  fn to_words(&self) -> [u32; WORDS] {
    let [c0, c1, c2, c3] = self.constants;
    let [k0, k1, k2, k3, k4, k5, k6, k7] = self.keys;
    let [n0, n1, n2] = self.nonces;
    [c0, c1, c2, c3, k0, k1, k2, k3, k4, k5, k6, k7, self.block_counter, n0, n1, n2]
  }
}

#[cfg_attr(test, derive(Debug))]
#[derive(Copy, Clone)]
struct Row(u32, u32, u32, u32);

impl Row {
  fn or(self, x: &Row) -> Row {
    Row(self.0 | x.0, self.1 | x.1, self.2 | x.2, self.3 | x.3)
  }

  fn roll_left<const N: u8>(self) -> Row {
    let lefted = self.shift_left::<N>();
    let righted = self.shift_right(32u8.wrapping_sub(N));
    lefted.or(&righted)
  }

  fn shift_left<const N: u8>(self) -> Row {
    Row(self.0 << N, self.1 << N, self.2 << N, self.3 << N)
  }

  fn shuffle_left1(self) -> Row {
    Row(self.1, self.2, self.3, self.0)
  }

  fn shuffle_left2(self) -> Row {
    Row(self.2, self.3, self.0, self.1)
  }

  fn shuffle_left3(self) -> Row {
    Row(self.3, self.0, self.1, self.2)
  }

  fn shift_right(self, bit_distance: u8) -> Row {
    Row(
      self.0 >> bit_distance,
      self.1 >> bit_distance,
      self.2 >> bit_distance,
      self.3 >> bit_distance,
    )
  }

  fn wrapping_add(self, x: &Row) -> Row {
    Row(
      self.0.wrapping_add(x.0),
      self.1.wrapping_add(x.1),
      self.2.wrapping_add(x.2),
      self.3.wrapping_add(x.3),
    )
  }

  fn xor(self, x: &Row) -> Row {
    Row(self.0 ^ x.0, self.1 ^ x.1, self.2 ^ x.2, self.3 ^ x.3)
  }
}

// https://datatracker.ietf.org/doc/html/rfc7539#section-2.3
fn block_function<const ADD: bool>(block: &Block) -> Block {
  let (mut a, mut b, mut c, mut d) = block.to_rows();

  for _ in 0..ITERATIONS {
    round(&mut a, &mut b, &mut c, &mut d);
    undiagonalize(&mut b, &mut c, &mut d);
    round(&mut a, &mut b, &mut c, &mut d);
    diagonalize(&mut b, &mut c, &mut d);
  }

  if ADD {
    let (e, f, g, h) = block.to_rows();
    a = a.wrapping_add(&e);
    b = b.wrapping_add(&f);
    c = c.wrapping_add(&g);
    d = d.wrapping_add(&h);
  }

  Block::from_rows(a, b, c, d)
}

fn diagonalize(b: &mut Row, c: &mut Row, d: &mut Row) {
  *b = b.shuffle_left3();
  *c = c.shuffle_left2();
  *d = d.shuffle_left1();
}

fn undiagonalize(b: &mut Row, c: &mut Row, d: &mut Row) {
  *b = b.shuffle_left1();
  *c = c.shuffle_left2();
  *d = d.shuffle_left3();
}

fn round(a: &mut Row, b: &mut Row, c: &mut Row, d: &mut Row) {
  *a = a.wrapping_add(b);
  *d = a.xor(d);
  *d = d.roll_left::<16>();

  *c = c.wrapping_add(d);
  *b = b.xor(c);
  *b = b.roll_left::<12>();

  *a = a.wrapping_add(b);
  *d = a.xor(d);
  *d = d.roll_left::<8>();

  *c = c.wrapping_add(d);
  *b = b.xor(c);
  *b = b.roll_left::<7>();
}

#[cfg(feature = "rand_core")]
mod rand_core {
  use crate::rng::{ChaCha20, Rng};

  impl rand_core::RngCore for ChaCha20 {
    fn next_u32(&mut self) -> u32 {
      u32::from_le_bytes(self.u8_4())
    }

    fn next_u64(&mut self) -> u64 {
      u64::from_le_bytes(self.u8_8())
    }

    fn fill_bytes(&mut self, dst: &mut [u8]) {
      self.fill_slice(dst);
    }
  }

  impl rand_core::CryptoRng for ChaCha20 {}
}

#[cfg(test)]
mod tests {
  use crate::rng::{
    Rng, SeedableRng,
    cha_cha20::{Block, ChaCha20, WORDS, block_function},
  };

  #[test]
  fn from_crescent_seeds() {
    let mut this = ChaCha20::from_seed([
      0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0,
      0,
    ])
    .unwrap();
    assert_eq!(u32::from_le_bytes(this.u8_4()), 137206642);
  }

  #[test]
  fn from_one_seeds() {
    let mut this = ChaCha20::from_seed([
      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
      1,
    ])
    .unwrap();
    for _ in 0..WORDS {
      let _ = this.u8_4();
    }
    let mut results = [0u32; WORDS];
    for elem in results.iter_mut() {
      *elem = u32::from_le_bytes(this.u8_4());
    }
    let expected = [
      0x2452eb3a, 0x9249f8ec, 0x8d829d9b, 0xddd4ceb1, 0xe8252083, 0x60818b01, 0xf38422b8,
      0x5aaa49c9, 0xbb00ca8e, 0xda3ba7b4, 0xc4b592d1, 0xfdf2732f, 0x4436274e, 0x2561b3c8,
      0xebdd4aa6, 0xa0136c00,
    ];
    assert_eq!(results, expected);
  }

  #[test]
  fn from_zero_seeds() {
    let mut this = ChaCha20::from_seed([0u8; 32]).unwrap();
    assert_eq!(
      {
        let mut array = [0; WORDS];
        for elem in &mut array {
          *elem = u32::from_le_bytes(this.u8_4());
        }
        array
      },
      [
        0xade0b876, 0x903df1a0, 0xe56a5d40, 0x28bd8653, 0xb819d2bd, 0x1aed8da0, 0xccef36a8,
        0xc70d778b, 0x7c5941da, 0x8d485751, 0x3fe02477, 0x374ad8b8, 0xf4b8436a, 0x1ca11815,
        0x69b687c3, 0x8665eeb2
      ]
    );
    assert_eq!(
      {
        let mut array = [0; WORDS];
        for elem in &mut array {
          *elem = u32::from_le_bytes(this.u8_4());
        }
        array
      },
      [
        0xbee7079f, 0x7a385155, 0x7c97ba98, 0x0d082d73, 0xa0290fcb, 0x6965e348, 0x3e53c612,
        0xed7aee32, 0x7621b729, 0x434ee69c, 0xb03371d5, 0xd539d874, 0x281fed31, 0x45fb0a51,
        0x1f0ae1ac, 0x6f4d794b
      ]
    );

    let mut from_crescent0 = ChaCha20::from_seed([
      0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0,
      0,
    ])
    .unwrap();
    assert_eq!(u32::from_le_bytes(from_crescent0.u8_4()), 137206642);
  }

  // https://datatracker.ietf.org/doc/html/rfc7539#section-2.3.2
  #[test]
  fn rfc_2_3_2() {
    let block = Block::from_words([
      0x61707865, 0x3320646e, 0x79622d32, 0x6b206574, 0x03020100, 0x07060504, 0x0b0a0908,
      0x0f0e0d0c, 0x13121110, 0x17161514, 0x1b1a1918, 0x1f1e1d1c, 0x00000001, 0x09000000,
      0x4a000000, 0x00000000,
    ]);
    assert_eq!(
      block_function::<false>(&block),
      Block::from_words([
        0x837778ab, 0xe238d763, 0xa67ae21e, 0x5950bb2f, 0xc4f2d0c7, 0xfc62bb2f, 0x8fa018fc,
        0x3f5ec7b7, 0x335271c2, 0xf29489f3, 0xeabda8fc, 0x82e46ebd, 0xd19c12b4, 0xb04e16de,
        0x9e83d0cb, 0x4e3c50a2,
      ])
    );
    assert_eq!(
      block_function::<true>(&block),
      Block::from_words([
        0xe4e7f110, 0x15593bd1, 0x1fdd0f50, 0xc47120a3, 0xc7f4d1c7, 0x0368c033, 0x9aaa2204,
        0x4e6cd4c3, 0x466482d2, 0x09aa9f07, 0x05d7c214, 0xa2028bd9, 0xd19c12b5, 0xb94e16de,
        0xe883d0cb, 0x4e3c50a2,
      ])
    );
  }
}
