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

impl<B> WeightedIndex<B>
where
  B: Clear + Lease<[B::Item]> + SingleTypeStorage + TryExtend<[B::Item; 1]>,
  B::Item: Clone + From<u8> + PartialOrd + TryArithmetic<Output = B::Item>,
{
  /// Creates a new instance with the given buffer and weights.
  #[inline]
  pub fn new(mut buffer: B, weights: impl IntoIterator<Item = B::Item>) -> crate::Result<Self> {
    buffer.clear();
    let mut sum = B::Item::from(0u8);
    for elem in weights {
      if elem < B::Item::from(0u8) {
        return Err(crate::Error::InvalidWeight);
      }
      sum = sum.try_add(elem)?;
      buffer.try_extend([sum.clone()])?;
    }
    Ok(Self { buffer, sum })
  }

  /// Mutable buffer
  #[inline]
  pub fn buffer_mut(&mut self) -> &mut B {
    &mut self.buffer
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
    B::Item: FromRng<R>,
    R: Rng,
  {
    let random = rng.pick_from_range(B::Item::from(0u8)..self.sum.clone())?;
    Some(self.buffer.lease().partition_point(|el| *el < random))
  }
}

impl<B> Default for WeightedIndex<B>
where
  B: Default + SingleTypeStorage,
  B::Item: Default,
{
  #[inline]
  fn default() -> Self {
    Self { buffer: B::default(), sum: B::Item::default() }
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    collection::ArrayVectorU8,
    rng::{SeedableRng, WeightedIndex, Xorshift64},
  };

  #[test]
  fn pick() {
    let mut buffer = ArrayVectorU8::<_, 3>::new();
    let mut rng = Xorshift64::from_std_random().unwrap();
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
}
