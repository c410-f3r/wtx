use core::{
  borrow::{Borrow, BorrowMut},
  ops::{Deref, DerefMut},
  slice::{Iter, IterMut},
};

/// Used for serialization/de-serialization.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct ArrayWrapper<T, const N: usize>(
  /// The actual array
  pub [T; N],
);

impl<T, const N: usize> AsRef<[T; N]> for ArrayWrapper<T, N> {
  #[inline]
  fn as_ref(&self) -> &[T; N] {
    self
  }
}

impl<T, const N: usize> AsMut<[T; N]> for ArrayWrapper<T, N> {
  #[inline]
  fn as_mut(&mut self) -> &mut [T; N] {
    self
  }
}

impl<T, const N: usize> Borrow<[T; N]> for ArrayWrapper<T, N> {
  #[inline]
  fn borrow(&self) -> &[T; N] {
    self
  }
}

impl<T, const N: usize> BorrowMut<[T; N]> for ArrayWrapper<T, N> {
  #[inline]
  fn borrow_mut(&mut self) -> &mut [T; N] {
    self
  }
}

impl<T, const N: usize> Default for ArrayWrapper<T, N>
where
  T: Default,
{
  #[inline]
  fn default() -> Self {
    ArrayWrapper(core::array::from_fn(|_| T::default()))
  }
}

impl<T, const N: usize> Deref for ArrayWrapper<T, N> {
  type Target = [T; N];

  #[inline]
  fn deref(&self) -> &[T; N] {
    &self.0
  }
}

impl<T, const N: usize> DerefMut for ArrayWrapper<T, N> {
  #[inline]
  fn deref_mut(&mut self) -> &mut [T; N] {
    &mut self.0
  }
}

impl<T, const N: usize> From<[T; N]> for ArrayWrapper<T, N> {
  #[inline]
  fn from(from: [T; N]) -> Self {
    Self(from)
  }
}

impl<'array, T, const N: usize> IntoIterator for &'array ArrayWrapper<T, N> {
  type IntoIter = Iter<'array, T>;
  type Item = &'array T;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.0.iter()
  }
}

impl<'array, T, const N: usize> IntoIterator for &'array mut ArrayWrapper<T, N> {
  type IntoIter = IterMut<'array, T>;
  type Item = &'array mut T;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.0.iter_mut()
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::collection::ArrayWrapper;
  use core::{fmt::Formatter, marker::PhantomData};
  use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, SeqAccess, Visitor},
    ser::SerializeTuple,
  };

  impl<'de, T, const N: usize> Deserialize<'de> for ArrayWrapper<T, N>
  where
    T: Default + Deserialize<'de>,
  {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      struct ArrayVisitor<T, const N: usize>(PhantomData<T>);

      impl<'de, T, const N: usize> Visitor<'de> for ArrayVisitor<T, N>
      where
        T: Default + Deserialize<'de>,
      {
        type Value = ArrayWrapper<T, N>;

        #[inline]
        fn expecting(&self, formatter: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
          formatter.write_fmt(format_args!("an array with {N} elements"))
        }

        #[inline]
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
          A: SeqAccess<'de>,
        {
          let mut counter: usize = 0;
          let mut rslt = ArrayWrapper::<T, N>::default();
          let mut iter = rslt.0.iter_mut();
          while let Some(deserialized) = seq.next_element::<T>()? {
            let Some(elem) = iter.next() else {
              return Err(de::Error::invalid_length(
                N,
                &"sequence has more data than what the array can hold",
              ));
            };
            *elem = deserialized;
            counter = counter.wrapping_add(1);
          }
          if counter != N {
            return Err(de::Error::invalid_length(N, &"array needs more data to be constructed"));
          }
          Ok(rslt)
        }
      }

      deserializer.deserialize_tuple(N, ArrayVisitor::<T, N>(PhantomData))
    }
  }

  impl<T, const N: usize> Serialize for ArrayWrapper<T, N>
  where
    T: Serialize,
  {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      let mut seq = serializer.serialize_tuple(N)?;
      for elem in &self.0 {
        seq.serialize_element(elem)?;
      }
      seq.end()
    }
  }
}
