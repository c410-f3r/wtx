#![expect(clippy::mem_forget, reason = "out-of-bounds elements are manually dropped")]

use crate::{
  collection::{
    ExpansionTy, LinearStorageLen,
    linear_storage::{
      LinearStorage, linear_storage_mut::LinearStorageMut, linear_storage_slice::LinearStorageSlice,
    },
    misc::drop_elements,
  },
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
/// [`ArrayVector`] with a capacity limited by `usize`.
pub type ArrayVectorUsize<T, const N: usize> = ArrayVector<usize, T, N>;

/// Errors of [`ArrayVector`].
#[derive(Debug)]
pub enum ArrayVectorError {
  /// Inner array is not totally full
  IntoInnerIncomplete,
  #[doc = doc_reserve_overflow!()]
  ReserveOverflow,
}

/// Storage backed by an arbitrary array.
pub struct ArrayVector<L, T, const N: usize>(Inner<L, T, N>)
where
  L: LinearStorageLen;

impl<L, T, const N: usize> ArrayVector<L, T, N>
where
  L: LinearStorageLen,
{
  const INSTANCE_CHECK: () = {
    assert!(N <= L::UPPER_BOUND_USIZE);
  };

  /// Constructs a new instance from a fully initialized array.
  #[inline]
  pub fn from_array(array: [T; N]) -> Self {
    const { Self::INSTANCE_CHECK };
    let mut this = Self::new();
    // `_INSTANCE_CHECK` makes this conversion infallible
    this.0.len = L::from_usize(N).unwrap_or_default();
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
    const { Self::INSTANCE_CHECK };
    // The initial check makes this conversion infallible.
    let data_len = L::from_usize(M).unwrap_or_default();
    let mut instance_len = data_len;
    if let Some(elem) = len {
      instance_len = instance_len.min(elem);
    }
    let mut this = Self::new();
    this.0.len = instance_len;
    // SAFETY: the inner `data` as well as the provided `data` have the same layout in different
    //         memory regions
    unsafe {
      ptr::copy_nonoverlapping(data.as_ptr(), this.as_ptr_mut(), instance_len.usize());
    }
    if Inner::<L, T, N>::NEEDS_DROP
      && let Some(diff) = data_len.checked_sub(instance_len)
      && diff > L::ZERO
    {
      // SAFETY: indices are within bounds
      unsafe {
        drop_elements(diff, instance_len, data.as_mut_ptr());
      }
    }
    mem::forget(data);
    this
  }

  /// Constructs a new empty instance.
  #[inline]
  pub const fn new() -> Self {
    const { Self::INSTANCE_CHECK };
    Self(Inner { len: L::ZERO, data: [const { MaybeUninit::uninit() }; N] })
  }

  /// Return the inner fixed size array, if the capacity is full.
  #[inline]
  pub fn into_inner(self) -> crate::Result<[T; N]> {
    if self.0.len.usize() < N {
      return Err(ArrayVectorError::IntoInnerIncomplete.into());
    }
    // SAFETY: All elements are initialized
    Ok(unsafe { ptr::read(self.0.data.as_ptr().cast()) })
  }

  unsafe fn get_owned(&mut self, idx: L) -> T {
    // SAFETY: it is up to the caller to provide a valid index
    let src = unsafe { self.0.data.as_ptr().add(idx.usize()) };
    // SAFETY: if the index is valid, then the element exists
    let elem = unsafe { ptr::read(src) };
    // SAFETY: if the index is valid, then the element is initialized
    unsafe { elem.assume_init() }
  }
}

impl<L, T, const N: usize> ArrayVector<L, T, N>
where
  L: LinearStorageLen,
{
  #[doc = from_cloneable_elem_doc!("ArrayVectorUsize::<_, 16>")]
  #[inline]
  pub fn from_cloneable_elem(len: usize, value: T) -> crate::Result<Self>
  where
    T: Clone,
  {
    Ok(Self(Inner::from_cloneable_elem(len, value)?))
  }

  #[doc = from_cloneable_slice_doc!("ArrayVectorUsize::<_, 16>")]
  #[inline]
  pub fn from_cloneable_slice(slice: &[T]) -> crate::Result<Self>
  where
    T: Clone,
  {
    Ok(Self(Inner::from_cloneable_slice(slice)?))
  }

  #[doc = from_copyable_slice_doc!("ArrayVectorUsize::<_, 16>")]
  #[inline]
  pub fn from_copyable_slice(slice: &[T]) -> crate::Result<Self>
  where
    T: Copy,
  {
    Ok(Self(Inner::from_copyable_slice(slice)?))
  }

  #[doc = from_iter_doc!("ArrayVectorUsize::<_, 16>", "[1, 2, 3]", "&[1, 2, 3]")]
  #[expect(clippy::should_implement_trait, reason = "The std trait is infallible")]
  #[inline]
  pub fn from_iter(iter: impl IntoIterator<Item = T>) -> crate::Result<Self> {
    Ok(Self(Inner::from_iter(iter)?))
  }

  #[doc = as_ptr_doc!("ArrayVectorUsize::<_, 16>", "[1, 2, 3]")]
  #[inline]
  pub fn as_ptr(&self) -> *const T {
    self.0.as_ptr()
  }

  #[doc = as_ptr_mut_doc!()]
  #[inline]
  pub fn as_ptr_mut(&mut self) -> *mut T {
    self.0.as_ptr_mut()
  }

  #[doc = as_slice_doc!("ArrayVectorUsize::<_, 16>", "[1, 2, 3]", "&[1, 2, 3]")]
  #[inline]
  pub fn as_slice(&self) -> &[T] {
    self.0.as_slice()
  }

  #[doc = as_slice_mut_doc!()]
  #[inline]
  pub fn as_slice_mut(&mut self) -> &mut [T] {
    self.0.as_slice_mut()
  }

  #[doc = capacity_doc!("ArrayVectorUsize::<_, 16>", "[1, 2, 3]")]
  #[inline]
  pub fn capacity(&self) -> L {
    self.0.capacity()
  }

  #[doc = clear_doc!("ArrayVectorUsize::<_, 16>", "[1, 2, 3]")]
  #[inline]
  pub fn clear(&mut self) {
    self.0.clear();
  }

  #[doc = expand_doc!("ArrayVectorUsize::<_, 16>")]
  #[inline]
  pub fn expand(&mut self, et: ExpansionTy, value: T) -> crate::Result<()>
  where
    T: Clone,
  {
    self.0.expand(et, value)
  }

  #[doc = extend_from_cloneable_slice_doc!("ArrayVectorUsize::<_, 16>")]
  #[inline]
  pub fn extend_from_cloneable_slice(&mut self, other: &[T]) -> crate::Result<()>
  where
    T: Clone,
  {
    self.0.extend_from_cloneable_slice(other)
  }

  #[doc = extend_from_copyable_slice_doc!("ArrayVectorUsize::<_, 16>")]
  #[inline]
  pub fn extend_from_copyable_slice(&mut self, other: &[T]) -> crate::Result<()>
  where
    T: Copy,
  {
    self.0.extend_from_copyable_slice(other)
  }

  #[doc = extend_from_copyable_slice_doc!("ArrayVectorUsize::<_, 16>")]
  #[inline]
  pub fn extend_from_copyable_slices<'iter, E, I>(&mut self, others: I) -> crate::Result<L>
  where
    E: Lease<[T]> + ?Sized + 'iter,
    I: IntoIterator<Item = &'iter E>,
    I::IntoIter: Clone,
    T: Copy + 'iter,
  {
    self.0.extend_from_copyable_slices(others)
  }

  #[doc = extend_from_iter_doc!("ArrayVectorUsize::<_, 16>", "[1, 2, 3]", "&[1, 2, 3]")]
  #[inline]
  pub fn extend_from_iter(&mut self, iter: impl IntoIterator<Item = T>) -> crate::Result<()> {
    self.0.extend_from_iter(iter)
  }

  #[doc = len_doc!()]
  #[inline]
  pub fn len(&self) -> L {
    self.0.len()
  }

  #[doc = pop_doc!("ArrayVectorUsize::<_, 16>", "[1, 2, 3]", "[1, 2]")]
  #[inline]
  pub fn pop(&mut self) -> Option<T> {
    <[T] as LinearStorageSlice>::pop(&mut self.0)
  }

  #[doc = push_doc!("ArrayVectorUsize::<_, 16>", "1", "&[1]")]
  #[inline]
  pub fn push(&mut self, elem: T) -> crate::Result<()> {
    self.0.push(elem)
  }

  #[doc = remaining_doc!("ArrayVectorUsize::<_, 16>", "1")]
  #[inline]
  pub fn remaining(&self) -> L {
    self.0.remaining()
  }

  #[doc = remove_doc!("ArrayVectorUsize::<_, 16>", "[1, 2, 3]", "[1, 3]")]
  #[inline]
  pub fn remove(&mut self, index: L) -> Option<T> {
    <[T] as LinearStorageSlice>::remove(&mut self.0, index)
  }

  #[doc = set_len_doc!()]
  #[inline]
  pub unsafe fn set_len(&mut self, new_len: L) {
    self.0.len = new_len;
  }

  #[doc = truncate_doc!("ArrayVectorUsize::<_, 16>", "[1, 2, 3]", "[1]")]
  #[inline]
  pub fn truncate(&mut self, new_len: L) {
    let _rslt = <[T] as LinearStorageSlice>::truncate(&mut self.0, new_len);
  }
}

impl<L, T, const N: usize> Clone for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
  T: Clone,
{
  #[inline]
  fn clone(&self) -> Self {
    let mut this = Self::new();
    let _rslt = this.0.extend_from_cloneable_slice(self);
    this
  }
}

impl<L, T, const N: usize> Debug for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
  T: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    self.lease().fmt(f)
  }
}

impl<L, T, const N: usize> Default for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<L, T, const N: usize> Deref for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
{
  type Target = [T];

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.0.as_slice()
  }
}

impl<L, T, const N: usize> DerefMut for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.0.as_slice_mut()
  }
}

impl<L, T, const N: usize> Drop for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn drop(&mut self) {
    if Inner::<L, T, N>::NEEDS_DROP {
      self.0.clear();
    }
  }
}

impl<L, T, const N: usize> Eq for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
  T: Eq,
{
}

impl<L, T, const N: usize> FromIterator<T> for Wrapper<crate::Result<ArrayVector<L, T, N>>>
where
  L: LinearStorageLen,
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
  L: LinearStorageLen,
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
  L: LinearStorageLen,
  T: 'any,
{
  type IntoIter = slice::Iter<'any, T>;
  type Item = &'any T;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.0.as_slice().iter()
  }
}

impl<'any, L, T, const N: usize> IntoIterator for &'any mut ArrayVector<L, T, N>
where
  L: LinearStorageLen,
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
  L: LinearStorageLen,
{
  #[inline]
  fn lease(&self) -> &[T] {
    self
  }
}

impl<L, T, const N: usize> LeaseMut<[T]> for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn lease_mut(&mut self) -> &mut [T] {
    self
  }
}

impl<L, T, const N: usize> PartialEq for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
  T: PartialEq,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

impl<L, T, const N: usize> PartialEq<[T]> for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
  T: PartialEq,
{
  #[inline]
  fn eq(&self, other: &[T]) -> bool {
    **self == *other
  }
}

impl<L, T, const N: usize> PartialOrd for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
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
  L: LinearStorageLen,
  T: Ord,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    (**self).cmp(other)
  }
}

impl<L, T, const N: usize> From<[T; N]> for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn from(from: [T; N]) -> Self {
    Self::from_parts(from, None)
  }
}

impl<'args, L, const N: usize> TryFrom<Arguments<'args>> for ArrayVector<L, u8, N>
where
  L: LinearStorageLen,
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
  L: LinearStorageLen,
  T: Clone,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &[T]) -> Result<Self, Self::Error> {
    let mut this = Self::new();
    this.0.extend_from_cloneable_slice(from)?;
    Ok(this)
  }
}

impl<L, const N: usize> fmt::Write for ArrayVector<L, u8, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn write_char(&mut self, c: char) -> fmt::Result {
    self
      .0
      .extend_from_copyable_slice(char_slice(&mut [0; 4], c).as_bytes())
      .map_err(|_err| fmt::Error)
  }

  #[inline]
  fn write_str(&mut self, s: &str) -> fmt::Result {
    self.0.extend_from_copyable_slice(s.as_bytes()).map_err(|_err| fmt::Error)
  }
}

#[cfg(feature = "std")]
impl<L, const N: usize> std::io::Write for ArrayVector<L, u8, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }

  #[inline]
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    let len = (self.0.remaining().usize()).min(buf.len());
    let _rslt = self.0.extend_from_copyable_slice(buf.get(..len).unwrap_or_default());
    Ok(len)
  }
}

/// A by-value array iterator.
#[derive(Debug)]
pub struct ArrayIntoIter<L, T, const N: usize>
where
  L: LinearStorageLen,
{
  idx: L,
  data: ArrayVector<L, T, N>,
}

impl<L, T, const N: usize> DoubleEndedIterator for ArrayIntoIter<L, T, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn next_back(&mut self) -> Option<Self::Item> {
    if let Some(diff) = self.data.0.len.checked_sub(L::ONE)
      && diff > L::ZERO
    {
      self.data.0.len = diff;
      // SAFETY: `diff` is within bounds
      return Some(unsafe { self.data.get_owned(diff) });
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
    let len = self.data.0.len;
    self.data.0.len = L::ZERO;
    if Inner::<L, T, N>::NEEDS_DROP {
      let diff = len.wrapping_sub(idx);
      if diff > L::ZERO {
        // SAFETY: indices are within bounds
        unsafe {
          drop_elements(diff, idx, self.data.as_ptr_mut());
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
    if self.idx >= self.data.0.len {
      return None;
    }
    let idx = self.idx;
    self.idx = idx.wrapping_add(L::ONE);
    // SAFETY: `idx` is within bounds
    Some(unsafe { self.data.get_owned(idx) })
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = self.data.0.len.wrapping_sub(self.idx);
    (len.usize(), Some(len.usize()))
  }
}

struct Inner<L, T, const N: usize>
where
  L: LinearStorageLen,
{
  len: L,
  data: [MaybeUninit<T>; N],
}

impl<L, T, const N: usize> Inner<L, T, N>
where
  L: LinearStorageLen,
{
  const INSTANCE_CHECK: () = {
    assert!(N <= L::UPPER_BOUND_USIZE);
  };
}

impl<L, T, const N: usize> LinearStorage<T> for Inner<L, T, N>
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

impl<L, T, const N: usize> LinearStorageMut<T> for Inner<L, T, N>
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

impl<L, T, const N: usize> Default for Inner<L, T, N>
where
  L: LinearStorageLen,
{
  fn default() -> Self {
    const { Self::INSTANCE_CHECK };
    Self { len: L::ZERO, data: [const { MaybeUninit::uninit() }; N] }
  }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
  use crate::{
    collection::{ArrayVector, LinearStorageLen},
    misc::Usize,
  };
  use arbitrary::{Arbitrary, Unstructured};

  impl<'any, L, T, const N: usize> Arbitrary<'any> for ArrayVector<L, T, N>
  where
    L: LinearStorageLen,
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
  use crate::collection::{ArrayVector, LinearStorageLen};
  use core::{fmt::Formatter, marker::PhantomData};
  use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, SeqAccess, Visitor},
  };

  impl<'de, L, T, const N: usize> Deserialize<'de> for ArrayVector<L, T, N>
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

      deserializer.deserialize_seq(LocalVisitor::<L, T, N>(PhantomData))
    }
  }

  impl<L, T, const N: usize> Serialize for ArrayVector<L, T, N>
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
