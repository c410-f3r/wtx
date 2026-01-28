// ****** A word is `u32` or 4 bytes *****
//
// `#[inline(always)]` made a huge positive difference in practically all places.

use crate::rng::{CryptoRng, Rng, SeedableRng};
use core::fmt::Debug;

#[cfg(all(not(feature = "_bench"), test))]
const BLOCKS_LEN: usize = 1;
#[cfg(not(all(not(feature = "_bench"), test)))]
const BLOCKS_LEN: usize = _simd! {
  4 => 1,
  16 => 4,
  32 => 8,
  // At the time of this writing, 16 didn't increased performance.
  // Probably not worth it. The stack would also be a lot larger.
  64 => 8
};
// 1 iteration = 2 rounds = 8 quarter rounds
const ITERATIONS: u8 = 10;
const TOTAL_WORDS: usize = BLOCKS_LEN * WORDS_PER_BLOCK;
const WORDS_PER_BLOCK: usize = 16;

/// `ChaCha` block function with 20 rounds and a nonce of 12 bytes as specified in
/// <https://datatracker.ietf.org/doc/html/rfc7539>.
///
/// This structure is `Copy` to allow usage with `AtomicCell` in concurrent scenarios.
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct ChaCha20 {
  block: ParBlock,
  idx: u16,
  output: ParBlock,
}

impl ChaCha20 {
  #[inline(always)]
  pub fn from_key(key: [u8; 32]) -> ChaCha20 {
    ChaCha20 {
      block: ParBlock::from_key_and_nonce(key, [0; 12]),
      // FIXME(STABLE): const { TOTAL_WORDS.into() }
      idx: TOTAL_WORDS as u16,
      output: ParBlock::new(),
    }
  }

  /// Creates a new instance where you are responsible for providing parameters.
  ///
  /// Ideally, `key` should have a high entropy.
  #[inline(always)]
  pub fn new(key: [u8; 32], nonce: [u8; 12]) -> ChaCha20 {
    ChaCha20 {
      block: ParBlock::from_key_and_nonce(key, nonce),
      // FIXME(STABLE): const { TOTAL_WORDS.into() }
      idx: TOTAL_WORDS as u16,
      output: ParBlock::new(),
    }
  }
}

impl CryptoRng for ChaCha20 {}

impl Rng for ChaCha20 {
  #[inline(always)]
  fn u8(&mut self) -> u8 {
    self.u8_4()[0]
  }

  #[inline(always)]
  fn u8_4(&mut self) -> [u8; 4] {
    if usize::from(self.idx) >= TOTAL_WORDS {
      block_function::<true>(&self.block, &mut self.output);
      self.idx = 0;
      self.block.increment_counter();
    }
    let idx = usize::from(self.idx);
    let block_idx = idx % BLOCKS_LEN;
    let word_idx = idx / BLOCKS_LEN;
    let rslt = self.output.0[word_idx].0[block_idx];
    self.idx = self.idx.wrapping_add(1);
    rslt.to_le_bytes()
  }

  #[inline(always)]
  fn u8_8(&mut self) -> [u8; 8] {
    let [a, b, c, d] = self.u8_4();
    let [e, f, g, h] = self.u8_4();
    [a, b, c, d, e, f, g, h]
  }

  #[inline(always)]
  fn u8_16(&mut self) -> [u8; 16] {
    let [a, b, c, d] = self.u8_4();
    let [e, f, g, h] = self.u8_4();
    let [i, j, k, l] = self.u8_4();
    let [m, n, o, p] = self.u8_4();
    [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p]
  }

  #[inline(always)]
  fn u8_32(&mut self) -> [u8; 32] {
    let [b0, b1, b2, b3] = self.u8_4();
    let [b4, b5, b6, b7] = self.u8_4();
    let [b8, b9, b10, b11] = self.u8_4();
    let [b12, b13, b14, b15] = self.u8_4();
    let [b16, b17, b18, b19] = self.u8_4();
    let [b20, b21, b22, b23] = self.u8_4();
    let [b24, b25, b26, b27] = self.u8_4();
    let [b28, b29, b30, b31] = self.u8_4();
    [
      b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16, b17, b18, b19,
      b20, b21, b22, b23, b24, b25, b26, b27, b28, b29, b30, b31,
    ]
  }
}

impl SeedableRng for ChaCha20 {
  type Seed = [u8; 32];

  #[inline(always)]
  fn from_seed(seed: Self::Seed) -> crate::Result<Self> {
    Ok(Self::from_key(seed))
  }
}

impl Debug for ChaCha20 {
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("ChaCha20").finish()
  }
}

/// Parallel Block
///
/// For one block made up of single words, 4 * 16 bytes = 64 bytes = 512 bits
#[cfg_attr(test, derive(Debug))]
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct ParBlock([ParWord; WORDS_PER_BLOCK]);

impl ParBlock {
  #[inline(always)]
  fn new() -> Self {
    Self([ParWord::default(); WORDS_PER_BLOCK])
  }

  #[cfg(all(not(feature = "_bench"), test))]
  #[inline(always)]
  fn from_words(words: [u32; WORDS_PER_BLOCK]) -> Self {
    let mut rslt = [ParWord::default(); WORDS_PER_BLOCK];
    for (from, to) in words.iter().zip(&mut rslt) {
      to.fill(*from);
    }
    Self(rslt)
  }

  #[inline(always)]
  fn from_key_and_nonce(key: [u8; 32], nonce: [u8; 12]) -> Self {
    #[rustfmt::skip]
    let [
      k0, k1, k2, k3, k4, k5, k6, k7, k8, k9, k10, k11, k12, k13, k14, k15,
      k16, k17, k18, k19, k20, k21, k22, k23, k24, k25, k26, k27, k28, k29, k30, k31,
    ] = key;
    let [n0, n1, n2, n3, n4, n5, n6, n7, n8, n9, n10, n11] = nonce;
    Self([
      ParWord::from_elem(0x6170_7865),
      ParWord::from_elem(0x3320_646E),
      ParWord::from_elem(0x7962_2D32),
      ParWord::from_elem(0x6B20_6574),
      ParWord::from_elem(u32::from_le_bytes([k0, k1, k2, k3])),
      ParWord::from_elem(u32::from_le_bytes([k4, k5, k6, k7])),
      ParWord::from_elem(u32::from_le_bytes([k8, k9, k10, k11])),
      ParWord::from_elem(u32::from_le_bytes([k12, k13, k14, k15])),
      ParWord::from_elem(u32::from_le_bytes([k16, k17, k18, k19])),
      ParWord::from_elem(u32::from_le_bytes([k20, k21, k22, k23])),
      ParWord::from_elem(u32::from_le_bytes([k24, k25, k26, k27])),
      ParWord::from_elem(u32::from_le_bytes([k28, k29, k30, k31])),
      ParWord::from_array({
        let mut idx: u32 = 0;
        core::array::from_fn(|_| {
          let rslt = idx;
          idx = idx.wrapping_add(1);
          rslt
        })
      }),
      ParWord::from_elem(u32::from_le_bytes([n0, n1, n2, n3])),
      ParWord::from_elem(u32::from_le_bytes([n4, n5, n6, n7])),
      ParWord::from_elem(u32::from_le_bytes([n8, n9, n10, n11])),
    ])
  }

  #[inline(always)]
  const fn block_counter_mut(&mut self) -> &mut ParWord {
    &mut self.0[12]
  }

  // https://datatracker.ietf.org/doc/html/rfc7539#section-2.4: ChaCha20 successively calls the
  // ChaCha20 block function, with the same key and nonce, and with successively increasing block
  // counter parameters.
  //
  // Not sure why an implementation would increase the nonces here.
  #[inline(always)]
  fn increment_counter(&mut self) {
    let blocks_len = ParWord::from_elem(BLOCKS_LEN as u32);
    *self.block_counter_mut() = self.block_counter_mut().wrapping_add(&blocks_len);
  }

  // https://datatracker.ietf.org/doc/html/rfc7539#section-2.1
  #[inline(always)]
  fn quarter_round(&mut self, a: usize, b: usize, c: usize, d: usize) {
    self.0[a] = self.0[a].wrapping_add(&self.0[b]);
    self.0[d] = self.0[d].xor(&self.0[a]);
    self.0[d] = self.0[d].rotate_left(16);

    self.0[c] = self.0[c].wrapping_add(&self.0[d]);
    self.0[b] = self.0[b].xor(&self.0[c]);
    self.0[b] = self.0[b].rotate_left(12);

    self.0[a] = self.0[a].wrapping_add(&self.0[b]);
    self.0[d] = self.0[d].xor(&self.0[a]);
    self.0[d] = self.0[d].rotate_left(8);

    self.0[c] = self.0[c].wrapping_add(&self.0[d]);
    self.0[b] = self.0[b].xor(&self.0[c]);
    self.0[b] = self.0[b].rotate_left(7);
  }
}

/// Parallel Word
///
/// Represents the word position of one or more blocks
#[cfg_attr(test, derive(Debug))]
#[derive(Clone, Copy, Default, Eq, Ord, PartialEq, PartialOrd)]
struct ParWord([u32; BLOCKS_LEN]);

impl ParWord {
  #[inline(always)]
  fn from_array(array: [u32; BLOCKS_LEN]) -> Self {
    Self(array)
  }

  #[inline(always)]
  fn from_elem(n: u32) -> Self {
    let mut rslt = Self::default();
    rslt.fill(n);
    rslt
  }

  #[inline(always)]
  fn fill(&mut self, n: u32) {
    self.0 = [n; BLOCKS_LEN];
  }

  #[inline(always)]
  fn rotate_left(&self, n: u32) -> Self {
    let mut rslt = self.0;
    for a in rslt.iter_mut() {
      *a = a.rotate_left(n);
    }
    Self(rslt)
  }

  #[inline(always)]
  fn wrapping_add(&self, other: &Self) -> Self {
    let mut rslt = self.0;
    for (a, b) in rslt.iter_mut().zip(other.0) {
      *a = a.wrapping_add(b);
    }
    Self(rslt)
  }

  #[inline(always)]
  fn xor(&self, other: &Self) -> Self {
    let mut rslt = self.0;
    for (a, b) in rslt.iter_mut().zip(other.0) {
      *a = *a ^ b;
    }
    Self(rslt)
  }
}

// https://datatracker.ietf.org/doc/html/rfc7539#section-2.3
#[inline(always)]
fn block_function<const ADD: bool>(block: &ParBlock, output: &mut ParBlock) {
  *output = *block;
  for _ in 0..ITERATIONS {
    output.quarter_round(0, 4, 8, 12);
    output.quarter_round(1, 5, 9, 13);
    output.quarter_round(2, 6, 10, 14);
    output.quarter_round(3, 7, 11, 15);
    output.quarter_round(0, 5, 10, 15);
    output.quarter_round(1, 6, 11, 12);
    output.quarter_round(2, 7, 8, 13);
    output.quarter_round(3, 4, 9, 14);
  }
  if ADD {
    for (a, b) in output.0.iter_mut().zip(&block.0) {
      *a = a.wrapping_add(b);
    }
  }
}

#[cfg(all(feature = "_bench", test))]
mod bench {
  use crate::rng::{ChaCha20, Rng};
  use core::hint::black_box;

  #[bench]
  fn cha_cha20(b: &mut test::Bencher) {
    let mut rng = ChaCha20::from_key([7; 32]);
    b.iter(|| {
      black_box({
        let mut array = [0; 1024 * 8];
        rng.shuffle_slice(&mut array);
      });
    });
  }
}

#[cfg(feature = "rand_core")]
mod rand_core {
  use crate::rng::{ChaCha20, Rng};

  impl rand_core::RngCore for ChaCha20 {
    #[inline(always)]
    fn next_u32(&mut self) -> u32 {
      u32::from_le_bytes(self.u8_4())
    }

    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
      u64::from_le_bytes(self.u8_8())
    }

    #[inline(always)]
    fn fill_bytes(&mut self, dst: &mut [u8]) {
      self.fill_slice(dst);
    }
  }

  impl rand_core::CryptoRng for ChaCha20 {}
}

#[cfg(all(not(feature = "_bench"), test))]
mod tests {
  use crate::rng::{
    Rng, SeedableRng,
    cha_cha20::{ChaCha20, ParBlock, WORDS_PER_BLOCK, block_function},
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
    for _ in 0..WORDS_PER_BLOCK {
      let _ = this.u8_4();
    }
    let mut results = [0u32; WORDS_PER_BLOCK];
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
        let mut array = [0; WORDS_PER_BLOCK];
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
        let mut array = [0; WORDS_PER_BLOCK];
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
    let block = ParBlock::from_words([
      0x61707865, 0x3320646e, 0x79622d32, 0x6b206574, 0x03020100, 0x07060504, 0x0b0a0908,
      0x0f0e0d0c, 0x13121110, 0x17161514, 0x1b1a1918, 0x1f1e1d1c, 0x00000001, 0x09000000,
      0x4a000000, 0x00000000,
    ]);
    let mut output = block;
    block_function::<false>(&block, &mut output);
    assert_eq!(
      output,
      ParBlock::from_words([
        0x837778ab, 0xe238d763, 0xa67ae21e, 0x5950bb2f, 0xc4f2d0c7, 0xfc62bb2f, 0x8fa018fc,
        0x3f5ec7b7, 0x335271c2, 0xf29489f3, 0xeabda8fc, 0x82e46ebd, 0xd19c12b4, 0xb04e16de,
        0x9e83d0cb, 0x4e3c50a2,
      ])
    );
    block_function::<true>(&block, &mut output);
    assert_eq!(
      output,
      ParBlock::from_words([
        0xe4e7f110, 0x15593bd1, 0x1fdd0f50, 0xc47120a3, 0xc7f4d1c7, 0x0368c033, 0x9aaa2204,
        0x4e6cd4c3, 0x466482d2, 0x09aa9f07, 0x05d7c214, 0xa2028bd9, 0xd19c12b5, 0xb94e16de,
        0xe883d0cb, 0x4e3c50a2,
      ])
    );
  }
}
