#![expect(clippy::mem_forget, reason = "out-of-bounds elements are manually dropped")]

use crate::{
  collection::{IndexedStorage, IndexedStorageLen, IndexedStorageMut},
  misc::{Lease, LeaseMut, Wrapper, char_slice},
};
use core::{
  cmp::Ordering,
  fmt::{self, Arguments, Debug, Formatter},
  iter::FusedIterator,
  mem::{self, MaybeUninit},
  ops::{Deref, DerefMut},
  ptr, slice,
};

/// [`ArrayVector`] with a capacity limited by `u8`.
pub type ArrayVectorU8<T, const N: usize> = ArrayVector<u8, T, N>;
/// [`ArrayVector`] with a capacity limited by `u16`.
pub type ArrayVectorU16<T, const N: usize> = ArrayVector<u16, T, N>;
/// [`ArrayVector`] with a capacity limited by `u32`.
pub type ArrayVectorU32<T, const N: usize> = ArrayVector<u32, T, N>;

/// Errors of [`ArrayVector`].
#[derive(Debug)]
pub enum ArrayVectorError {
  /// Inner array is not totally full
  IntoInnerIncomplete,
  #[doc = doc_reserve_overflow!()]
  ReserveOverflow,
}

/// Storage backed by an arbitrary array.
pub struct ArrayVector<L, T, const N: usize>
where
  L: IndexedStorageLen,
{
  len: L,
  data: [MaybeUninit<T>; N],
}

impl<L, T, const N: usize> ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
{
  const _INSTANCE_CHECK: () = {
    assert!(N <= L::UPPER_BOUND_USIZE);
  };

  /// Constructs a new instance from a fully initialized array.
  #[inline]
  pub fn from_array(array: [T; N]) -> Self {
    let mut this = Self::new();
    // `_INSTANCE_CHECK` makes this conversion infallible
    this.len = L::from_usize(N).unwrap_or_default();
    // SAFETY: the inner `data` as well as the provided `array` have the same layout in different
    //         memory regions
    unsafe {
      ptr::copy_nonoverlapping(array.as_ptr(), this.as_ptr_mut(), N);
    }
    mem::forget(array);
    this
  }

  /// Constructs a new instance reusing `data` elements optionally delimited by `len`.
  ///
  /// The actual length will be the smallest value among `M`, `N` and `len`
  #[inline]
  pub fn from_parts<const M: usize>(mut data: [T; M], len: Option<L>) -> Self {
    const {
      assert!(M <= L::UPPER_BOUND_USIZE);
      assert!(M <= N);
    }
    // The initial check makes this conversion infallible.
    let data_len = L::from_usize(M).unwrap_or_default();
    let mut instance_len = data_len;
    if let Some(elem) = len {
      instance_len = instance_len.min(elem);
    }
    let mut this = Self::new();
    this.len = instance_len;
    // SAFETY: the inner `data` as well as the provided `data` have the same layout in different
    //         memory regions
    unsafe {
      ptr::copy_nonoverlapping(data.as_ptr(), this.as_ptr_mut(), instance_len.usize());
    }
    if Self::NEEDS_DROP
      && let Some(diff) = data_len.checked_sub(instance_len)
      && diff > L::ZERO
    {
      // SAFETY: indices are within bounds
      unsafe {
        Self::drop_elements(diff, instance_len, data.as_mut_ptr());
      }
    }
    mem::forget(data);
    this
  }

  /// Constructs a new empty instance.
  #[inline]
  pub const fn new() -> Self {
    Self { len: L::ZERO, data: [const { MaybeUninit::uninit() }; N] }
  }

  /// Return the inner fixed size array, if the capacity is full.
  #[inline]
  pub fn into_inner(self) -> crate::Result<[T; N]> {
    if self.len.usize() < N {
      return Err(ArrayVectorError::IntoInnerIncomplete.into());
    }
    // SAFETY: All elements are initialized
    Ok(unsafe { ptr::read(self.data.as_ptr().cast()) })
  }

  unsafe fn drop_elements(len: L, offset: L, ptr: *mut T) {
    // SAFETY: it is up to the caller to provide a valid pointer with a valid index
    let data = unsafe { ptr.add(offset.usize()) };
    // SAFETY: it is up to the caller to provide a valid length
    let elements = unsafe { slice::from_raw_parts_mut(data, len.usize()) };
    // SAFETY: it is up to the caller to provide parameters that can lead to droppable elements
    unsafe {
      ptr::drop_in_place(elements);
    }
  }

  unsafe fn get_owned(&mut self, idx: L) -> T {
    // SAFETY: it is up to the caller to provide a valid index
    let src = unsafe { self.data.as_ptr().add(idx.usize()) };
    // SAFETY: if the index is valid, then the element exists
    let elem = unsafe { ptr::read(src) };
    // SAFETY: if the index is valid, then the element is initialized
    unsafe { elem.assume_init() }
  }
}

impl<L, T, const N: usize> IndexedStorage<T> for ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
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

impl<L, T, const N: usize> IndexedStorageMut<T> for ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn as_ptr_mut(&mut self) -> *mut T {
    self.data.as_mut_ptr().cast()
  }

  #[inline]
  fn pop(&mut self) -> Option<T> {
    let new_len = self.len.checked_sub(L::ONE)?;
    self.len = new_len;
    // SAFETY: `new_len` is within bounds
    Some(unsafe { self.get_owned(new_len) })
  }

  #[inline]
  fn reserve(&mut self, additional: Self::Len) -> crate::Result<()> {
    if additional > self.remaining() {
      return Err(ArrayVectorError::ReserveOverflow.into());
    }
    Ok(())
  }

  #[inline]
  unsafe fn set_len(&mut self, new_len: Self::Len) {
    self.len = new_len;
  }

  #[inline]
  fn truncate(&mut self, new_len: Self::Len) {
    let len = self.len;
    let diff = if let Some(diff) = len.checked_sub(new_len)
      && diff > L::ZERO
    {
      diff
    } else {
      return;
    };
    self.len = new_len;
    if Self::NEEDS_DROP {
      // SAFETY: indices are within bounds
      unsafe {
        Self::drop_elements(diff, new_len, self.as_ptr_mut());
      }
    }
  }
}

impl<L, T, const N: usize> Clone for ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
  T: Clone,
{
  #[inline]
  fn clone(&self) -> Self {
    let mut this = Self::new();
    let _rslt = this.extend_from_cloneable_slice(self);
    this
  }
}

impl<L, T, const N: usize> Debug for ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
  T: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    self.lease().fmt(f)
  }
}

impl<L, T, const N: usize> Default for ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<L, T, const N: usize> Deref for ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
{
  type Target = [T];

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.as_slice()
  }
}

impl<L, T, const N: usize> DerefMut for ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.as_slice_mut()
  }
}

impl<L, T, const N: usize> Drop for ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn drop(&mut self) {
    if Self::NEEDS_DROP {
      self.clear();
    }
  }
}

impl<L, T, const N: usize> Eq for ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
  T: Eq,
{
}

impl<L, T, const N: usize> FromIterator<T> for Wrapper<crate::Result<ArrayVector<L, T, N>>>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn from_iter<I>(iter: I) -> Self
  where
    I: IntoIterator<Item = T>,
  {
    Wrapper(ArrayVector::from_iter(iter))
  }
}

impl<L, T, const N: usize> IntoIterator for ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
{
  type IntoIter = ArrayIntoIter<L, T, N>;
  type Item = T;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    ArrayIntoIter { idx: L::ZERO, data: self }
  }
}

impl<'any, L, T, const N: usize> IntoIterator for &'any ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
  T: 'any,
{
  type IntoIter = slice::Iter<'any, T>;
  type Item = &'any T;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.as_slice().iter()
  }
}

impl<'any, L, T, const N: usize> IntoIterator for &'any mut ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
  T: 'any,
{
  type IntoIter = slice::IterMut<'any, T>;
  type Item = &'any mut T;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.iter_mut()
  }
}

impl<L, T, const N: usize> Lease<[T]> for ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn lease(&self) -> &[T] {
    self
  }
}

impl<L, T, const N: usize> LeaseMut<[T]> for ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn lease_mut(&mut self) -> &mut [T] {
    self
  }
}

impl<L, T, const N: usize> PartialEq for ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
  T: PartialEq,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

impl<L, T, const N: usize> PartialEq<[T]> for ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
  T: PartialEq,
{
  #[inline]
  fn eq(&self, other: &[T]) -> bool {
    **self == *other
  }
}

impl<L, T, const N: usize> PartialOrd for ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
  T: PartialOrd,
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

impl<L, T, const N: usize> Ord for ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
  T: Ord,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    (**self).cmp(other)
  }
}

impl<L, T, const N: usize> From<[T; N]> for ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn from(from: [T; N]) -> Self {
    Self::from_parts(from, None)
  }
}

impl<'args, L, const N: usize> TryFrom<Arguments<'args>> for ArrayVector<L, u8, N>
where
  L: IndexedStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(from: Arguments<'args>) -> Result<Self, Self::Error> {
    let mut rslt = Self::new();
    fmt::Write::write_fmt(&mut rslt, from)?;
    Ok(rslt)
  }
}

impl<L, T, const N: usize> TryFrom<&[T]> for ArrayVector<L, T, N>
where
  L: IndexedStorageLen,
  T: Clone,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &[T]) -> Result<Self, Self::Error> {
    let mut this = Self::new();
    this.extend_from_cloneable_slice(from)?;
    Ok(this)
  }
}

impl<L, const N: usize> fmt::Write for ArrayVector<L, u8, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn write_char(&mut self, c: char) -> fmt::Result {
    self
      .extend_from_copyable_slice(char_slice(&mut [0; 4], c).as_bytes())
      .map_err(|_err| fmt::Error)
  }

  #[inline]
  fn write_str(&mut self, s: &str) -> fmt::Result {
    self.extend_from_copyable_slice(s.as_bytes()).map_err(|_err| fmt::Error)
  }
}

#[cfg(feature = "std")]
impl<L, const N: usize> std::io::Write for ArrayVector<L, u8, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }

  #[inline]
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    let len = (self.remaining().usize()).min(buf.len());
    let _rslt = self.extend_from_copyable_slice(buf.get(..len).unwrap_or_default());
    Ok(len)
  }
}

/// A by-value array iterator.
#[derive(Debug)]
pub struct ArrayIntoIter<L, T, const N: usize>
where
  L: IndexedStorageLen,
{
  idx: L,
  data: ArrayVector<L, T, N>,
}

impl<L, T, const N: usize> DoubleEndedIterator for ArrayIntoIter<L, T, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn next_back(&mut self) -> Option<Self::Item> {
    if let Some(diff) = self.data.len.checked_sub(L::ONE)
      && diff > L::ZERO
    {
      self.data.len = diff;
      // SAFETY: `diff` is within bounds
      return Some(unsafe { self.data.get_owned(diff) });
    }
    None
  }
}

impl<L, T, const N: usize> Drop for ArrayIntoIter<L, T, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn drop(&mut self) {
    let idx = self.idx;
    let len = self.data.len;
    self.data.len = L::ZERO;
    if ArrayVector::<L, T, N>::NEEDS_DROP {
      let diff = len.wrapping_sub(idx);
      if diff > L::ZERO {
        // SAFETY: indices are within bounds
        unsafe {
          ArrayVector::<L, T, N>::drop_elements(diff, idx, self.data.as_ptr_mut());
        }
      }
    }
  }
}

impl<L, T, const N: usize> ExactSizeIterator for ArrayIntoIter<L, T, N> where L: IndexedStorageLen {}

impl<L, T, const N: usize> FusedIterator for ArrayIntoIter<L, T, N> where L: IndexedStorageLen {}

impl<L, T, const N: usize> Iterator for ArrayIntoIter<L, T, N>
where
  L: IndexedStorageLen,
{
  type Item = T;

  #[inline]
  fn count(self) -> usize {
    self.len()
  }

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.idx >= self.data.len {
      return None;
    }
    let idx = self.idx;
    self.idx = idx.wrapping_add(L::ONE);
    // SAFETY: `idx` is within bounds
    Some(unsafe { self.data.get_owned(idx) })
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = self.data.len.wrapping_sub(self.idx);
    (len.usize(), Some(len.usize()))
  }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
  use crate::{
    collection::{ArrayVector, IndexedStorageLen, IndexedStorageMut as _},
    misc::Usize,
  };
  use arbitrary::{Arbitrary, Unstructured};

  impl<'any, L, T, const N: usize> Arbitrary<'any> for ArrayVector<L, T, N>
  where
    L: IndexedStorageLen,
    T: Arbitrary<'any>,
  {
    #[inline]
    fn arbitrary(u: &mut Unstructured<'any>) -> arbitrary::Result<Self> {
      let mut len = const {
        let [_, _, _, _, a, b, c, d] = Usize::from_usize(N).into_u64().to_be_bytes();
        u32::from_be_bytes([a, b, c, d])
      };
      len = u32::arbitrary(u)?.min(len);
      let mut this = Self::new();
      for _ in 0..len {
        let _rslt = this.push(T::arbitrary(u)?);
      }
      Ok(this)
    }
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::collection::{ArrayVector, IndexedStorageLen, IndexedStorageMut};
  use core::{fmt::Formatter, marker::PhantomData};
  use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, SeqAccess, Visitor},
    ser::SerializeTuple,
  };

  impl<'de, L, T, const N: usize> Deserialize<'de> for ArrayVector<L, T, N>
  where
    L: IndexedStorageLen,
    T: Deserialize<'de>,
  {
    #[inline]
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
      DE: Deserializer<'de>,
    {
      struct ArrayVisitor<L, T, const N: usize>(PhantomData<(L, T)>);

      impl<'de, L, T, const N: usize> Visitor<'de> for ArrayVisitor<L, T, N>
      where
        L: IndexedStorageLen,
        T: Deserialize<'de>,
      {
        type Value = ArrayVector<L, T, N>;

        #[inline]
        fn expecting(&self, formatter: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
          formatter.write_fmt(format_args!("a vector with at most {N} elements"))
        }

        #[inline]
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
          A: SeqAccess<'de>,
        {
          let mut this = ArrayVector::new();
          while let Some(elem) = seq.next_element()? {
            this.push(elem).map_err(|_err| {
              de::Error::invalid_length(N, &"vector need more data to be constructed")
            })?;
          }
          Ok(this)
        }
      }

      deserializer.deserialize_seq(ArrayVisitor::<L, T, N>(PhantomData))
    }
  }

  impl<L, T, const N: usize> Serialize for ArrayVector<L, T, N>
  where
    L: IndexedStorageLen,
    T: Serialize,
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
