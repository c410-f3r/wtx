use crate::misc::{char_slice, Lease, LeaseMut, Usize};
use core::{
  array,
  cmp::Ordering,
  fmt::{self, Debug, Formatter},
  iter,
  mem::needs_drop,
  ops::{Deref, DerefMut},
  slice,
};

/// Errors of [`ArrayVector`].
#[derive(Debug)]
pub enum ArrayVectorError {
  #[doc = doc_many_elems_cap_overflow!()]
  ExtendFromSliceOverflow,
  #[doc = doc_single_elem_cap_overflow!()]
  PushOverflow,
}

/// A wrapper around the std's vector with some additional methods to manipulate copyable data.
pub struct ArrayVector<D, const N: usize> {
  len: u32,
  data: [D; N],
}

impl<D, const N: usize> ArrayVector<D, N> {
  /// Constructs a new instance reusing any `data` elements delimited by `len`.
  #[inline]
  pub const fn new(data: [D; N], len: u32) -> Self {
    let n = const {
      assert!(N <= Usize::from_u32(u32::MAX).into_usize() && !needs_drop::<D>());
      let [_, _, _, _, a, b, c, d] = Usize::from_usize(N).into_u64().to_be_bytes();
      u32::from_be_bytes([a, b, c, d])
    };
    Self { len: if len > n { n } else { len }, data }
  }

  /// The number of elements that can be stored.
  #[inline]
  pub fn capacity(&self) -> u32 {
    const {
      let [_, _, _, _, a, b, c, d] = Usize::from_usize(N).into_u64().to_be_bytes();
      u32::from_be_bytes([a, b, c, d])
    }
  }

  /// Clears the vector, removing all values.
  #[inline]
  pub fn clear(&mut self) {
    self.len = 0;
  }

  /// Shortens the vector, removing the last element.
  #[inline]
  pub fn pop(&mut self) -> bool {
    if let Some(elem) = self.len.checked_sub(1) {
      self.len = elem;
      true
    } else {
      false
    }
  }

  /// How many elements can be added to this collection.
  #[inline]
  pub fn remaining(&self) -> u32 {
    self.capacity().wrapping_sub(self.len)
  }

  /// Appends an element to the back of the collection.
  #[inline]
  pub fn push(&mut self, value: D) -> Result<(), ArrayVectorError> {
    if let Some(elem) = self.data.get_mut(*Usize::from(self.len)) {
      *elem = value;
      self.len = self.len.wrapping_add(1);
      Ok(())
    } else {
      Err(ArrayVectorError::PushOverflow)
    }
  }

  /// Shortens the vector, keeping the first `len` elements.
  #[inline]
  pub fn truncate(&mut self, len: u32) {
    self.len = len.min(self.capacity());
  }
}

impl<D, const N: usize> ArrayVector<D, N>
where
  D: Copy,
{
  /// Iterates over the slice `other`, copies each element, and then appends
  /// it to this vector. The `other` slice is traversed in-order.
  #[inline]
  pub fn extend_from_slice(&mut self, other: &[D]) -> Result<(), ArrayVectorError> {
    let Some(len) = u32::try_from(other.len()).ok().filter(|el| self.remaining() >= *el) else {
      return Err(ArrayVectorError::ExtendFromSliceOverflow);
    };
    let begin = *Usize::from(self.len);
    let end = *Usize::from(self.len.wrapping_add(len));
    self.data.get_mut(begin..end).unwrap_or_default().copy_from_slice(other);
    self.len = self.len.wrapping_add(len);
    Ok(())
  }
}

impl<D, const N: usize> Clone for ArrayVector<D, N>
where
  D: Clone,
{
  #[inline]
  fn clone(&self) -> Self {
    Self { data: self.data.clone(), len: self.len }
  }
}

impl<D, const N: usize> Debug for ArrayVector<D, N>
where
  D: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    self.lease().fmt(f)
  }
}

impl<D, const N: usize> Default for ArrayVector<D, N>
where
  D: Default,
{
  #[inline]
  fn default() -> Self {
    Self::new(array::from_fn(|_| D::default()), 0)
  }
}

impl<D, const N: usize> Deref for ArrayVector<D, N> {
  type Target = [D];

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.data.get(..*Usize::from(self.len)).unwrap_or_default()
  }
}

impl<D, const N: usize> DerefMut for ArrayVector<D, N> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.data.get_mut(..*Usize::from(self.len)).unwrap_or_default()
  }
}

impl<D, const N: usize> Eq for ArrayVector<D, N> where D: Eq {}

impl<D, const N: usize> IntoIterator for ArrayVector<D, N> {
  type IntoIter = iter::Take<array::IntoIter<D, N>>;
  type Item = D;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.data.into_iter().take(*Usize::from(self.len))
  }
}

impl<'any, D, const N: usize> IntoIterator for &'any ArrayVector<D, N>
where
  D: 'any,
{
  type IntoIter = slice::Iter<'any, D>;
  type Item = &'any D;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

impl<'any, D, const N: usize> IntoIterator for &'any mut ArrayVector<D, N>
where
  D: 'any,
{
  type IntoIter = slice::IterMut<'any, D>;
  type Item = &'any mut D;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.iter_mut()
  }
}

impl<D, const N: usize> Lease<[D]> for ArrayVector<D, N> {
  #[inline]
  fn lease(&self) -> &[D] {
    self
  }
}

impl<D, const N: usize> LeaseMut<[D]> for ArrayVector<D, N> {
  #[inline]
  fn lease_mut(&mut self) -> &mut [D] {
    self
  }
}

impl<D, const N: usize> PartialEq for ArrayVector<D, N>
where
  D: PartialEq,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

impl<D, const N: usize> PartialEq<[D]> for ArrayVector<D, N>
where
  D: PartialEq,
{
  #[inline]
  fn eq(&self, other: &[D]) -> bool {
    **self == *other
  }
}

impl<D, const N: usize> PartialOrd for ArrayVector<D, N>
where
  D: PartialOrd,
{
  #[inline]
  fn ge(&self, other: &Self) -> bool {
    (**self).ge(&**other)
  }

  #[inline]
  fn gt(&self, other: &Self) -> bool {
    (**self).gt(&**other)
  }

  #[inline]
  fn le(&self, other: &Self) -> bool {
    (**self).le(&**other)
  }

  #[inline]
  fn lt(&self, other: &Self) -> bool {
    (**self).lt(&**other)
  }

  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    (**self).partial_cmp(&**other)
  }
}

impl<D, const N: usize> Ord for ArrayVector<D, N>
where
  D: Ord,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    (**self).cmp(other)
  }
}

impl<D, const N: usize> TryFrom<&[D]> for ArrayVector<D, N>
where
  D: Copy + Default,
{
  type Error = ArrayVectorError;

  #[inline]
  fn try_from(from: &[D]) -> Result<Self, Self::Error> {
    let mut this = Self::default();
    this.extend_from_slice(from)?;
    Ok(this)
  }
}

impl<const N: usize> fmt::Write for ArrayVector<u8, N> {
  #[inline]
  fn write_char(&mut self, c: char) -> fmt::Result {
    self.extend_from_slice(char_slice(&mut [0; 4], c)).map_err(|_err| fmt::Error)
  }

  #[inline]
  fn write_str(&mut self, s: &str) -> fmt::Result {
    self.extend_from_slice(s.as_bytes()).map_err(|_err| fmt::Error)
  }
}

#[cfg(feature = "std")]
impl<const N: usize> std::io::Write for ArrayVector<u8, N> {
  #[inline]
  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }

  #[inline]
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    let len = (*Usize::from(self.remaining())).min(buf.len());
    let _rslt = self.extend_from_slice(buf.get(..len).unwrap_or_default());
    Ok(len)
  }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
  use crate::misc::{ArrayVector, Usize};
  use arbitrary::{Arbitrary, Unstructured};

  impl<'any, T, const N: usize> Arbitrary<'any> for ArrayVector<T, N>
  where
    T: Default + Arbitrary<'any>,
  {
    #[inline]
    fn arbitrary(u: &mut Unstructured<'any>) -> arbitrary::Result<Self> {
      let mut len = const {
        let [_, _, _, _, a, b, c, d] = Usize::from_usize(N).into_u64().to_be_bytes();
        u32::from_be_bytes([a, b, c, d])
      };
      len = u32::arbitrary(u)?.min(len);
      let mut data = core::array::from_fn(|_| T::default());
      for elem in data.iter_mut().take(*Usize::from(len)) {
        *elem = T::arbitrary(u)?;
      }
      Ok(Self { len, data })
    }
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::misc::ArrayVector;
  use core::{fmt::Formatter, marker::PhantomData};
  use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeTuple,
    Deserialize, Deserializer, Serialize, Serializer,
  };

  impl<'de, D, const N: usize> Deserialize<'de> for ArrayVector<D, N>
  where
    D: Default + Deserialize<'de>,
  {
    #[inline]
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
      DE: Deserializer<'de>,
    {
      struct ArrayVisitor<D, const N: usize>(PhantomData<D>);

      impl<'de, D, const N: usize> Visitor<'de> for ArrayVisitor<D, N>
      where
        D: Default + Deserialize<'de>,
      {
        type Value = ArrayVector<D, N>;

        #[inline]
        fn expecting(&self, formatter: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
          formatter.write_fmt(format_args!("an array with {N} elements"))
        }

        #[inline]
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
          A: SeqAccess<'de>,
        {
          let mut this = ArrayVector::default();
          for elem in &mut this {
            *elem = seq.next_element::<D>()?.ok_or_else(|| {
              de::Error::invalid_length(N, &"Array need more data to be constructed")
            })?;
          }
          Ok(this)
        }
      }

      deserializer.deserialize_tuple(N, ArrayVisitor::<D, N>(PhantomData))
    }
  }

  impl<D, const N: usize> Serialize for ArrayVector<D, N>
  where
    D: Serialize,
  {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      let mut seq = serializer.serialize_tuple(N)?;
      for elem in self.iter() {
        seq.serialize_element(elem)?;
      }
      seq.end()
    }
  }
}
