use crate::collections::{
  ArrayVectorError, LinearStorageLen,
  linear_storage::{LinearStorage, linear_storage_mut::LinearStorageMut},
  misc::drop_elements,
};
use core::{
  iter::FusedIterator,
  mem::{ManuallyDrop, MaybeUninit},
  ptr, slice,
};

/// A by-value array iterator.
#[derive(Debug)]
pub struct ArrayIntoIter<L, T, const N: usize>
where
  L: LinearStorageLen,
{
  array: ArrayVectorInner<L, T, N>,
  idx: L,
}

impl<L, T, const N: usize> DoubleEndedIterator for ArrayIntoIter<L, T, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn next_back(&mut self) -> Option<Self::Item> {
    if let Ok(diff) = self.array.len.try_sub(L::ONE)
      && diff >= self.idx
    {
      self.array.len = diff;
      // SAFETY: `diff` is within bounds
      return Some(unsafe { get_owned(&self.array.data, diff) });
    }
    None
  }
}

impl<L, T, const N: usize> Drop for ArrayIntoIter<L, T, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn drop(&mut self) {
    let idx = self.idx;
    let len = self.array.len;
    self.array.len = L::ZERO;
    if ArrayVectorInner::<L, T, N>::NEEDS_DROP {
      let diff = len.wrapping_sub(idx);
      if diff > L::ZERO {
        // SAFETY: indices are within bounds
        unsafe {
          let _rslt = drop_elements(&mut (), diff, idx, self.array.data.as_mut_ptr());
        }
      }
    }
  }
}

impl<L, T, const N: usize> ExactSizeIterator for ArrayIntoIter<L, T, N> where L: LinearStorageLen {}

impl<L, T, const N: usize> FusedIterator for ArrayIntoIter<L, T, N> where L: LinearStorageLen {}

impl<L, T, const N: usize> Iterator for ArrayIntoIter<L, T, N>
where
  L: LinearStorageLen,
{
  type Item = T;

  #[inline]
  fn count(self) -> usize {
    self.len()
  }

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.idx >= self.array.len {
      return None;
    }
    let idx = self.idx;
    self.idx = idx.wrapping_add(L::ONE);
    // SAFETY: `idx` is within bounds
    Some(unsafe { get_owned(&self.array.data, idx) })
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = self.array.len.wrapping_sub(self.idx);
    (len.usize(), Some(len.usize()))
  }
}

#[derive(Debug)]
pub(crate) struct ArrayVectorInner<L, T, const N: usize>
where
  L: LinearStorageLen,
{
  pub(crate) len: L,
  pub(crate) data: [MaybeUninit<T>; N],
}

impl<L, T, const N: usize> ArrayVectorInner<L, T, N>
where
  L: LinearStorageLen,
{
  const INSTANCE_CHECK: () = {
    assert!(N <= L::UPPER_BOUND_USIZE);
  };

  #[inline]
  pub(crate) const fn new() -> Self {
    const { Self::INSTANCE_CHECK };
    Self { len: L::ZERO, data: [const { MaybeUninit::uninit() }; N] }
  }

  #[inline]
  pub(crate) fn as_inner(&self) -> crate::Result<&[T; N]> {
    if self.len.usize() != N {
      return Err(ArrayVectorError::IntoInnerIncomplete.into());
    }
    // SAFETY: All elements are initialized
    Ok(unsafe { &*self.data.as_ptr().cast() })
  }

  #[inline]
  pub(crate) fn into_inner(self) -> crate::Result<[T; N]> {
    if self.len.usize() < N {
      drop(self.into_iter());
      return Err(ArrayVectorError::IntoInnerIncomplete.into());
    }
    let this = ManuallyDrop::new(self);
    // SAFETY: All elements are initialized
    Ok(unsafe { ptr::read(this.data.as_ptr().cast()) })
  }
}

impl<L, T, const N: usize> LinearStorage<T> for ArrayVectorInner<L, T, N>
where
  L: LinearStorageLen,
{
  type Len = L;
  type Slice = [T];

  #[inline]
  fn as_ptr(&self) -> *const T {
    self.data.as_ptr().cast()
  }

  #[inline]
  fn capacity(&self) -> Self::Len {
    L::from_usize(N).unwrap_or_default()
  }

  #[inline]
  fn len(&self) -> Self::Len {
    self.len
  }
}

impl<L, T, const N: usize> LinearStorageMut<T> for ArrayVectorInner<L, T, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn as_ptr_mut(&mut self) -> *mut T {
    self.data.as_mut_ptr().cast()
  }

  #[inline]
  fn reserve(&mut self, additional: Self::Len) -> crate::Result<()> {
    if additional > self.remaining() {
      return Err(ArrayVectorError::ReserveOverflow.into());
    }
    Ok(())
  }

  #[inline]
  fn reserve_exact(&mut self, additional: Self::Len) -> crate::Result<()> {
    if additional > self.remaining() {
      return Err(ArrayVectorError::ReserveOverflow.into());
    }
    Ok(())
  }

  #[inline]
  unsafe fn set_len(&mut self, new_len: Self::Len) {
    self.len = new_len;
  }
}

#[allow(clippy::expl_impl_clone_on_copy, reason = "`MaybeUninit` does not implement Clone")]
impl<L, T, const N: usize> Clone for ArrayVectorInner<L, T, N>
where
  L: LinearStorageLen,
  T: Clone,
{
  #[inline]
  fn clone(&self) -> Self {
    let mut this = Self::new();
    let _rslt = this.extend_from_cloneable_slice(self.as_slice());
    this
  }
}

impl<L, T, const N: usize> Copy for ArrayVectorInner<L, T, N>
where
  L: LinearStorageLen,
  T: Copy,
{
}

impl<L, T, const N: usize> Default for ArrayVectorInner<L, T, N>
where
  L: LinearStorageLen,
{
  fn default() -> Self {
    const { Self::INSTANCE_CHECK };
    Self { len: L::ZERO, data: [const { MaybeUninit::uninit() }; N] }
  }
}

impl<L, T, const N: usize> IntoIterator for ArrayVectorInner<L, T, N>
where
  L: LinearStorageLen,
{
  type IntoIter = ArrayIntoIter<L, T, N>;
  type Item = T;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    ArrayIntoIter { array: self, idx: L::ZERO }
  }
}

impl<'any, L, T, const N: usize> IntoIterator for &'any ArrayVectorInner<L, T, N>
where
  L: LinearStorageLen,
  T: 'any,
{
  type IntoIter = slice::Iter<'any, T>;
  type Item = &'any T;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.as_slice().iter()
  }
}

impl<'any, L, T, const N: usize> IntoIterator for &'any mut ArrayVectorInner<L, T, N>
where
  L: LinearStorageLen,
  T: 'any,
{
  type IntoIter = slice::IterMut<'any, T>;
  type Item = &'any mut T;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.as_slice_mut().iter_mut()
  }
}

unsafe fn get_owned<L, T, const N: usize>(data: &[MaybeUninit<T>; N], idx: L) -> T
where
  L: LinearStorageLen,
{
  // SAFETY: it is up to the caller to provide a valid index
  let src = unsafe { data.as_ptr().add(idx.usize()) };
  // SAFETY: if the index is valid, then the element exists
  let elem = unsafe { ptr::read(src) };
  // SAFETY: if the index is valid, then the element is initialized
  unsafe { elem.assume_init() }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::collections::{
    LinearStorageLen, array_vector_inner::ArrayVectorInner,
    linear_storage::linear_storage_mut::LinearStorageMut as _,
  };
  use core::{fmt::Formatter, marker::PhantomData};
  use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, SeqAccess, Visitor},
  };

  impl<'de, L, T, const N: usize> Deserialize<'de> for ArrayVectorInner<L, T, N>
  where
    L: LinearStorageLen,
    T: Deserialize<'de>,
  {
    #[inline]
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
      DE: Deserializer<'de>,
    {
      struct LocalVisitor<L, T, const N: usize>(PhantomData<(L, T)>);

      impl<'de, L, T, const N: usize> Visitor<'de> for LocalVisitor<L, T, N>
      where
        L: LinearStorageLen,
        T: Deserialize<'de>,
      {
        type Value = ArrayVectorInner<L, T, N>;

        #[inline]
        fn expecting(&self, formatter: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
          formatter.write_fmt(format_args!("a vector with at most {N} elements"))
        }

        #[inline]
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
          A: SeqAccess<'de>,
        {
          let mut this = ArrayVectorInner::new();
          while let Some(elem) = seq.next_element()? {
            this.push(elem).map_err(de::Error::custom)?;
          }
          Ok(this)
        }
      }

      deserializer.deserialize_seq(LocalVisitor::<L, T, N>(PhantomData))
    }
  }

  impl<L, T, const N: usize> Serialize for ArrayVectorInner<L, T, N>
  where
    L: LinearStorageLen,
    T: Serialize,
  {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      serializer.collect_seq(self)
    }
  }
}
