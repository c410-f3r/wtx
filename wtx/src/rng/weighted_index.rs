use crate::{
  collection::{Clear, TryExtend},
  misc::{Lease, SingleTypeStorage, TryArithmetic},
  rng::{FromRng, Rng},
};

/// Allows the random selection of an element in a list where each element has a proportional
/// chance of being selected based on its assigned weight.
#[derive(Debug)]
pub struct WeightedIndex<B>
where
  B: SingleTypeStorage,
{
  buffer: B,
  sum: B::Item,
}

impl<B, E> WeightedIndex<B>
where
  B: Clear + Lease<[E]> + SingleTypeStorage<Item = E> + TryExtend<[E; 1]>,
  E: Clone + From<u8> + PartialOrd + TryArithmetic<Output = E>,
{
  /// Creates a new instance with the given buffer and weights.
  ///
  /// No element in the `weights` set should be negative.
  #[inline]
  pub fn new(buffer: B, weights: impl IntoIterator<Item = E>) -> crate::Result<Self> {
    let mut this = Self { buffer, sum: E::from(0u8) };
    this.recalc(weights)?;
    Ok(this)
  }

  /// Mutable buffer
  #[inline]
  pub fn buffer_mut(&mut self) -> &mut B {
    &mut self.buffer
  }

  /// Clears internal state
  #[inline]
  pub fn clear(&mut self) {
    let Self { buffer, sum } = self;
    buffer.clear();
    *sum = E::from(0u8);
  }

  /// Buffer ownership
  #[inline]
  pub fn into_buffer(self) -> B {
    self.buffer
  }

  /// Picks a random weighted index
  #[inline]
  pub fn pick<R>(&self, rng: &mut R) -> Option<usize>
  where
    E: FromRng<R>,
    R: Rng,
  {
    let buffer = self.buffer.lease();
    let len = buffer.len();
    let sum = self.sum.clone();
    let Some(random) = rng.pick_from_range(E::from(0u8)..sum) else {
      if buffer.is_empty() {
        return None;
      }
      return Some(0);
    };
    let idx = buffer.partition_point(|el| *el <= random);
    Some(idx.min(len.wrapping_sub(1)))
  }

  /// Clears previous results to calculate new weights using the same internal buffer.
  ///
  /// No element in the `weights` set should be negative.
  #[inline]
  pub fn recalc(&mut self, weights: impl IntoIterator<Item = E>) -> crate::Result<()> {
    self.clear();
    let Self { buffer, sum } = self;
    for elem in weights {
      if elem < E::from(0u8) {
        return Err(crate::Error::InvalidWeight);
      }
      *sum = sum.try_add(elem)?;
      buffer.try_extend([sum.clone()])?;
    }
    Ok(())
  }
}

impl<B, E> Default for WeightedIndex<B>
where
  B: Default + SingleTypeStorage<Item = E>,
  E: Default,
{
  #[inline]
  fn default() -> Self {
    Self { buffer: B::default(), sum: E::default() }
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    collection::ArrayVectorU8,
    rng::{SeedableRng, WeightedIndex, Xorshift64},
  };

  #[test]
  fn distribution() {
    let mut buffer = ArrayVectorU8::<_, 3>::new();
    let mut rng = Xorshift64::from_simple_seed().unwrap();
    let weighted_index = WeightedIndex::new(&mut buffer, [6, 2, 12]).unwrap();
    assert_eq!(weighted_index.buffer, &[6, 8, 20]);
    let mut indices = [0, 0, 0];
    for _ in 0..10000 {
      let idx = weighted_index.pick(&mut rng).unwrap();
      indices[idx] += 1;
    }
    assert!(indices[0] > indices[1]);
    assert!(indices[2] > indices[0] && indices[2] > indices[1]);
  }

  #[test]
  fn zeros() {
    let mut buffer = ArrayVectorU8::<_, 3>::new();
    let mut rng = Xorshift64::from_simple_seed().unwrap();
    let weighted_index = WeightedIndex::new(&mut buffer, [0, 0, 0]).unwrap();
    assert_eq!(weighted_index.buffer, &[0, 0, 0]);
    assert_eq!(weighted_index.pick(&mut rng).unwrap(), 0);
  }
}
