use crate::{
  collections::{
    ArrayIntoIter, ExpansionTy, LinearStorageLen,
    array_vector_inner::ArrayVectorInner,
    linear_storage::{
      LinearStorage as _, linear_storage_mut::LinearStorageMut as _,
      linear_storage_slice::LinearStorageSlice,
    },
    misc::drop_elements,
  },
  misc::{Lease, LeaseMut, Wrapper, char_slice},
};
use core::{
  cmp::Ordering,
  fmt::{self, Arguments, Debug, Formatter},
  hash::{Hash, Hasher},
  mem::ManuallyDrop,
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
#[derive(Clone, Copy, Debug)]
pub enum ArrayVectorError {
  /// Inner array is not totally full
  IntoInnerIncomplete,
  #[doc = doc_reserve_overflow!()]
  ReserveOverflow,
}

/// Storage backed by an arbitrary array.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
  feature = "serde",
  serde(bound(serialize = "T: serde::Serialize", deserialize = "T: serde::Deserialize<'de>"))
)]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct ArrayVector<L, T, const N: usize>(ArrayVectorInner<L, T, N>)
where
  L: LinearStorageLen;

impl<L, T, const N: usize> ArrayVector<L, T, N>
where
  L: LinearStorageLen,
{
  /// See [`Self::capacity`].
  pub const CAPACITY: usize = N;

  const INSTANCE_CHECK: () = {
    assert!(N <= L::UPPER_BOUND_USIZE);
  };

  /// Constructs a new instance from a fully initialized array.
  #[inline]
  pub fn from_array<const M: usize>(array: [T; M]) -> Self {
    Self::from_parts(array, None)
  }

  /// Constructs a new instance reusing `data` elements optionally delimited by `len`.
  ///
  /// The actual length will be the smallest value among `M`, `N` and `len`
  #[inline]
  pub fn from_parts<const M: usize>(data: [T; M], len: Option<L>) -> Self {
    const { Self::INSTANCE_CHECK };
    const {
      assert!(M <= L::UPPER_BOUND_USIZE);
      assert!(M <= N);
    }
    let mut data_md = ManuallyDrop::new(data);
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
      ptr::copy_nonoverlapping(data_md.as_ptr(), this.as_ptr_mut(), instance_len.usize());
    }
    if ArrayVectorInner::<L, T, N>::NEEDS_DROP
      && let Ok(diff) = data_len.try_sub(instance_len)
      && diff > L::ZERO
    {
      // SAFETY: indices are within bounds
      unsafe {
        let _rslt = drop_elements(&mut (), diff, instance_len, data_md.as_mut_ptr());
      }
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
    let md = ManuallyDrop::new(self);
    // SAFETY: The `into_inner` method already handles drop elements
    let inner: ArrayVectorInner<L, T, N> = unsafe { ptr::read(ptr::addr_of!(md.0)) };
    inner.into_inner()
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
    Ok(Self(ArrayVectorInner::from_cloneable_elem(len, value)?))
  }

  #[doc = from_cloneable_slice_doc!("ArrayVectorUsize::<_, 16>")]
  #[inline]
  pub fn from_cloneable_slice(slice: &[T]) -> crate::Result<Self>
  where
    T: Clone,
  {
    Ok(Self(ArrayVectorInner::from_cloneable_slice(slice)?))
  }

  #[doc = from_copyable_slice_doc!("ArrayVectorUsize::<_, 16>")]
  #[inline]
  pub fn from_copyable_slice(slice: &[T]) -> crate::Result<Self>
  where
    T: Copy,
  {
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
  pub fn extend_from_copyable_slices<E, I>(&mut self, others: I) -> crate::Result<L>
  where
    E: Lease<[T]>,
    I: IntoIterator<Item = E>,
    I::IntoIter: Clone,
    T: Copy,
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
    // SAFETY: Up to the caller
    unsafe { self.0.set_len(new_len) }
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
    if ArrayVectorInner::<L, T, N>::NEEDS_DROP {
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
    Wrapper(ArrayVector::from_iterator(iter))
  }
}

impl<L, T, const N: usize> Hash for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
  T: Hash,
{
  #[inline]
  fn hash<H>(&self, state: &mut H)
  where
    H: Hasher,
  {
    Hash::hash(&**self, state);
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
    let md = ManuallyDrop::new(self);
    // SAFETY: The `into_iter` method already handles drop elements. See `ArrayIntoIter`.
    let inner: ArrayVectorInner<L, T, N> = unsafe { ptr::read(ptr::addr_of!(md.0)) };
    inner.into_iter()
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

impl<L, T, U, const N: usize> PartialEq<[U]> for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &[U]) -> bool {
    **self == *other
  }
}
impl<L, T, U, const N: usize> PartialEq<&[U]> for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&[U]) -> bool {
    **self == **other
  }
}
impl<L, T, U, const N: usize> PartialEq<&mut [U]> for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&mut [U]) -> bool {
    **self == **other
  }
}

impl<L, T, U, const M: usize, const N: usize> PartialEq<[U; M]> for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &[U; M]) -> bool {
    **self == *other
  }
}
impl<L, T, U, const M: usize, const N: usize> PartialEq<&[U; M]> for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&[U; M]) -> bool {
    **self == **other
  }
}
impl<L, T, U, const M: usize, const N: usize> PartialEq<&mut [U; M]> for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&mut [U; M]) -> bool {
    **self == **other
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
    (**self).cmp(&**other)
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
