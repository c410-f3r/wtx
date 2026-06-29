use crate::{
  collections::{
    ArrayIntoIter, ExpansionTy, LinearStorageLen as _,
    array_vector_inner::ArrayVectorInner,
    linear_storage::{
      LinearStorage as _, linear_storage_mut::LinearStorageMut as _,
      linear_storage_slice::LinearStorageSlice,
    },
  },
  misc::{Lease, LeaseMut, Wrapper, char_slice},
};
use core::{
  cmp::Ordering,
  fmt::{self, Arguments, Debug, Formatter},
  hash::{Hash, Hasher},
  ops::{Deref, DerefMut},
  ptr, slice,
};

/// Storage backed by an arbitrary array where elements are copyable.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Clone, Copy)]
pub struct ArrayVectorCopy<T, const N: usize>(ArrayVectorInner<u8, T, N>)
where
  T: Copy;

impl<T, const N: usize> ArrayVectorCopy<T, N>
where
  T: Copy,
{
  /// See [`Self::capacity`].
  pub const CAPACITY: usize = N;

  const INSTANCE_CHECK: () = {
    assert!(N <= u8::UPPER_BOUND_USIZE);
  };

  /// Constructs a new instance from a fully initialized array.
  #[inline]
  pub const fn from_array<const M: usize>(array: [T; M]) -> Self {
    Self::from_parts(array, None)
  }

  /// Constructs a new instance reusing `data` elements optionally delimited by `len`.
  ///
  /// The actual length will be the smallest value among `M`, `N` and `len`
  #[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "conversions are infallible"
  )]
  #[inline]
  pub const fn from_parts<const M: usize>(data: [T; M], len: Option<u8>) -> Self {
    const { Self::INSTANCE_CHECK };
    const {
      assert!(M <= u8::UPPER_BOUND_USIZE);
      assert!(M <= N);
    }
    let data_len = M as u8;
    let mut instance_len = data_len;
    if let Some(elem) = len
      && elem < instance_len
    {
      instance_len = elem;
    }
    let mut this = Self::new();
    this.0.len = instance_len;
    // SAFETY: the inner `data` as well as the provided `data` have the same layout in different
    //         memory regions
    unsafe {
      ptr::copy_nonoverlapping(
        data.as_ptr(),
        this.0.data.as_mut_ptr().cast(),
        instance_len as usize,
      );
    }
    this
  }

  /// Constructs a new empty instance.
  #[inline]
  pub const fn new() -> Self {
    const { Self::INSTANCE_CHECK };
    Self(ArrayVectorInner::new())
  }

  /// Returns the inner fixed size array, if the capacity is full.
  #[inline]
  pub fn as_inner(&self) -> crate::Result<&[T; N]> {
    self.0.as_inner()
  }

  /// Owned version of [`Self::as_inner`].
  #[inline]
  pub fn into_inner(self) -> crate::Result<[T; N]> {
    self.0.into_inner()
  }
}

impl<T, const N: usize> ArrayVectorCopy<T, N>
where
  T: Copy,
{
  #[doc = from_copyable_slice_doc!("ArrayVectorUsize::<_, 16>")]
  #[inline]
  pub fn from_copyable_slice(slice: &[T]) -> crate::Result<Self> {
    Ok(Self(ArrayVectorInner::from_copyable_slice(slice)?))
  }

  #[doc = from_iter_doc!("ArrayVectorUsize::<_, 16>", "[1, 2, 3]", "&[1, 2, 3]")]
  #[inline]
  pub fn from_iterator(iter: impl IntoIterator<Item = T>) -> crate::Result<Self> {
    Ok(Self(ArrayVectorInner::from_iterator(iter)?))
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
  pub fn capacity(&self) -> u8 {
    self.0.capacity()
  }

  #[doc = clear_doc!("ArrayVectorUsize::<_, 16>", "[1, 2, 3]")]
  #[inline]
  pub fn clear(&mut self) {
    self.0.clear();
  }

  #[doc = expand_doc!("ArrayVectorUsize::<_, 16>")]
  #[inline]
  pub fn expand(&mut self, et: ExpansionTy, value: T) -> crate::Result<()> {
    self.0.expand(et, value)
  }

  #[doc = extend_from_copyable_slice_doc!("ArrayVectorUsize::<_, 16>")]
  #[inline]
  pub fn extend_from_copyable_slice(&mut self, other: &[T]) -> crate::Result<()> {
    self.0.extend_from_copyable_slice(other)
  }

  #[doc = extend_from_copyable_slice_doc!("ArrayVectorUsize::<_, 16>")]
  #[inline]
  pub fn extend_from_copyable_slices<E, I>(&mut self, others: I) -> crate::Result<u8>
  where
    E: Lease<[T]>,
    I: IntoIterator<Item = E>,
    I::IntoIter: Clone,
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
  pub fn len(&self) -> u8 {
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
  pub fn remaining(&self) -> u8 {
    self.0.remaining()
  }

  #[doc = remove_doc!("ArrayVectorUsize::<_, 16>", "[1, 2, 3]", "[1, 3]")]
  #[inline]
  pub fn remove(&mut self, index: u8) -> Option<T> {
    <[T] as LinearStorageSlice>::remove(&mut self.0, index)
  }

  #[doc = set_len_doc!()]
  #[inline]
  pub unsafe fn set_len(&mut self, new_len: u8) {
    // SAFETY: Up to the caller
    unsafe { self.0.set_len(new_len) }
  }

  #[doc = truncate_doc!("ArrayVectorUsize::<_, 16>", "[1, 2, 3]", "[1]")]
  #[inline]
  pub fn truncate(&mut self, new_len: u8) {
    let _rslt = <[T] as LinearStorageSlice>::truncate(&mut self.0, new_len);
  }
}

impl<T, const N: usize> Debug for ArrayVectorCopy<T, N>
where
  T: Copy + Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    self.lease().fmt(f)
  }
}

impl<T, const N: usize> Default for ArrayVectorCopy<T, N>
where
  T: Copy,
{
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<T, const N: usize> Deref for ArrayVectorCopy<T, N>
where
  T: Copy,
{
  type Target = [T];

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.0.as_slice()
  }
}

impl<T, const N: usize> DerefMut for ArrayVectorCopy<T, N>
where
  T: Copy,
{
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.0.as_slice_mut()
  }
}

impl<T, const N: usize> Eq for ArrayVectorCopy<T, N> where T: Copy + Eq {}

impl<T, const N: usize> FromIterator<T> for Wrapper<crate::Result<ArrayVectorCopy<T, N>>>
where
  T: Copy,
{
  #[inline]
  fn from_iter<I>(iter: I) -> Self
  where
    I: IntoIterator<Item = T>,
  {
    Wrapper(ArrayVectorCopy::from_iterator(iter))
  }
}

impl<T, const N: usize> Hash for ArrayVectorCopy<T, N>
where
  T: Copy + Hash,
{
  #[inline]
  fn hash<H>(&self, state: &mut H)
  where
    H: Hasher,
  {
    Hash::hash(&**self, state);
  }
}

impl<T, const N: usize> IntoIterator for ArrayVectorCopy<T, N>
where
  T: Copy,
{
  type IntoIter = ArrayIntoIter<u8, T, N>;
  type Item = T;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

impl<'any, T, const N: usize> IntoIterator for &'any ArrayVectorCopy<T, N>
where
  T: Copy + 'any,
{
  type IntoIter = slice::Iter<'any, T>;
  type Item = &'any T;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.0.as_slice().iter()
  }
}

impl<'any, T, const N: usize> IntoIterator for &'any mut ArrayVectorCopy<T, N>
where
  T: Copy + 'any,
{
  type IntoIter = slice::IterMut<'any, T>;
  type Item = &'any mut T;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.iter_mut()
  }
}

impl<T, const N: usize> Lease<[T]> for ArrayVectorCopy<T, N>
where
  T: Copy,
{
  #[inline]
  fn lease(&self) -> &[T] {
    self
  }
}

impl<T, const N: usize> LeaseMut<[T]> for ArrayVectorCopy<T, N>
where
  T: Copy,
{
  #[inline]
  fn lease_mut(&mut self) -> &mut [T] {
    self
  }
}

impl<T, const N: usize> PartialEq for ArrayVectorCopy<T, N>
where
  T: Copy + PartialEq,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

impl<T, U, const N: usize> PartialEq<[U]> for ArrayVectorCopy<T, N>
where
  T: Copy + PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &[U]) -> bool {
    **self == *other
  }
}
impl<T, U, const N: usize> PartialEq<&[U]> for ArrayVectorCopy<T, N>
where
  T: Copy + PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&[U]) -> bool {
    **self == **other
  }
}
impl<T, U, const N: usize> PartialEq<&mut [U]> for ArrayVectorCopy<T, N>
where
  T: Copy + PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&mut [U]) -> bool {
    **self == **other
  }
}

impl<T, U, const M: usize, const N: usize> PartialEq<[U; M]> for ArrayVectorCopy<T, N>
where
  T: Copy + PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &[U; M]) -> bool {
    **self == *other
  }
}
impl<T, U, const M: usize, const N: usize> PartialEq<&[U; M]> for ArrayVectorCopy<T, N>
where
  T: Copy + PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&[U; M]) -> bool {
    **self == **other
  }
}
impl<T, U, const M: usize, const N: usize> PartialEq<&mut [U; M]> for ArrayVectorCopy<T, N>
where
  T: Copy + PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&mut [U; M]) -> bool {
    **self == **other
  }
}

impl<T, const N: usize> PartialOrd for ArrayVectorCopy<T, N>
where
  T: Copy + PartialOrd,
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

impl<T, const N: usize> Ord for ArrayVectorCopy<T, N>
where
  T: Copy + Ord,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    (**self).cmp(&**other)
  }
}

impl<T, const N: usize> From<[T; N]> for ArrayVectorCopy<T, N>
where
  T: Copy,
{
  #[inline]
  fn from(from: [T; N]) -> Self {
    Self::from_parts(from, None)
  }
}

impl<'args, const N: usize> TryFrom<Arguments<'args>> for ArrayVectorCopy<u8, N> {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: Arguments<'args>) -> Result<Self, Self::Error> {
    let mut rslt = Self::new();
    fmt::Write::write_fmt(&mut rslt, from)?;
    Ok(rslt)
  }
}

impl<T, const N: usize> TryFrom<&[T]> for ArrayVectorCopy<T, N>
where
  T: Copy,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &[T]) -> Result<Self, Self::Error> {
    let mut this = Self::new();
    this.0.extend_from_copyable_slice(from)?;
    Ok(this)
  }
}

impl<const N: usize> fmt::Write for ArrayVectorCopy<u8, N> {
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
impl<const N: usize> std::io::Write for ArrayVectorCopy<u8, N> {
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
