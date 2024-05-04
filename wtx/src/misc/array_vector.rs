use crate::misc::{char_slice, Lease, Usize};
use core::{
  cmp::Ordering,
  fmt::{self, Debug, Formatter},
  ops::{Deref, DerefMut},
  slice,
};

/// A wrapper around the std's vector with some additional methods to manipulate copyable data.
pub struct ArrayVector<D, const N: usize> {
  len: u32,
  data: [D; N],
}

impl<D, const N: usize> ArrayVector<D, N>
where
  D: Copy,
{
  /// Constructs a new instance reusing any `data` elements delimited by `len`.
  #[allow(
    // False positive
    clippy::missing_panics_doc
  )]
  #[inline]
  pub const fn new(data: [D; N], len: u32) -> Self {
    let n = const {
      if N > Usize::from_u32(u32::MAX).into_usize() {
        panic!("Capacity is too large");
      }
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

  /// Clears the vector, removing all values.
  #[inline]
  pub fn into_inner(self) -> impl Iterator<Item = D> {
    self.data.into_iter().take(*Usize::from(self.len))
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

  /// Iterates over the slice `other`, copies each element, and then appends
  /// it to this vector. The `other` slice is traversed in-order.
  #[inline]
  pub fn try_extend_from_slice(&mut self, other: &[D]) -> crate::Result<()> {
    let Some(len) = u32::try_from(other.len()).ok().filter(|el| self.remaining() >= *el) else {
      return Err(crate::Error::CapacityOverflow);
    };
    let begin = *Usize::from(self.len);
    let end = *Usize::from(self.len.wrapping_add(len));
    self.data.get_mut(begin..end).unwrap_or_default().copy_from_slice(other);
    self.len = self.len.wrapping_add(len);
    Ok(())
  }

  /// Appends an element to the back of the collection.
  #[inline]
  pub fn try_push(&mut self, value: D) -> crate::Result<()> {
    if let Some(elem) = self.data.get_mut(*Usize::from(self.len)) {
      *elem = value;
      self.len = self.len.wrapping_add(1);
      Ok(())
    } else {
      Err(crate::Error::CapacityOverflow)
    }
  }

  /// Shortens the vector, keeping the first `len` elements.
  #[inline]
  pub fn truncate(&mut self, len: u32) {
    self.len = len.min(self.capacity());
  }
}

impl<D, const N: usize> Clone for ArrayVector<D, N>
where
  D: Copy,
{
  #[inline]
  fn clone(&self) -> Self {
    Self { data: self.data, len: self.len }
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
  D: Copy + Default,
{
  #[inline]
  fn default() -> Self {
    Self::new([D::default(); N], 0)
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
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &[D]) -> Result<Self, Self::Error> {
    let mut this = Self::default();
    this.try_extend_from_slice(from)?;
    Ok(this)
  }
}

impl<const N: usize> fmt::Write for ArrayVector<u8, N> {
  #[inline]
  fn write_char(&mut self, ch: char) -> fmt::Result {
    self.try_extend_from_slice(char_slice(&mut [0; 4], ch)).map_err(|_err| fmt::Error)
  }

  #[inline]
  fn write_str(&mut self, str: &str) -> fmt::Result {
    self.try_extend_from_slice(str.as_bytes()).map_err(|_err| fmt::Error)
  }
}

#[cfg(feature = "std")]
impl<const N: usize> std::io::Write for ArrayVector<u8, N> {
  #[inline]
  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }

  #[inline]
  fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
    let len = (*Usize::from(self.remaining())).min(data.len());
    let _rslt = self.try_extend_from_slice(data.get(..len).unwrap_or_default());
    Ok(len)
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::misc::ArrayVector;
  use serde::{ser::SerializeTuple, Serialize, Serializer};

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