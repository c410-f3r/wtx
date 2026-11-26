mod dyn_vector_data;
mod dyn_vector_inner;

use crate::{
  collections::{
    ExpansionTy, SparseLinearStorageLen, Truncate, U7Len, U15Len, U31Len,
    linear_storage::{
      LinearStorage as _, linear_storage_mut::LinearStorageMut as _,
      linear_storage_slice::LinearStorageSlice,
    },
  },
  misc::{Lease, LeaseMut, char_slice, unlikely_unreachable},
};
use core::{
  cmp::Ordering,
  fmt::{self, Arguments, Debug, Formatter},
  hash::{Hash, Hasher},
  mem::MaybeUninit,
  ops::{Deref, DerefMut},
  slice,
};

/// [`DynVector`] with a capacity limited by `u7`.
pub type DynVectorU7<T, const N: usize> = DynVector<U7Len, T, N>;
/// [`DynVector`] with a capacity limited by `u15`.
pub type DynVectorU15<T, const N: usize> = DynVector<U15Len, T, N>;
/// [`DynVector`] with a capacity limited by `u31`.
pub type DynVectorU31<T, const N: usize> = DynVector<U31Len, T, N>;

/// Errors of [`DynVector`].
#[derive(Clone, Copy, Debug)]
pub enum DynVectorError {
  /// The index provided in the `insert` method is out of bounds.
  OutOfBoundsInsertIdx,
  #[doc = doc_reserve_overflow!()]
  ReserveOverflow {
    /// Additional
    additional: u16,
    /// Current
    curr: u32,
    /// Maximum
    max: u32,
  },
}

/// A `Vector`-like container that can store a small number of elements in the stack.
pub struct DynVector<L, T, const N: usize>(dyn_vector_inner::DynVectorInner<L, T, N>)
where
  L: SparseLinearStorageLen;

impl<L, T, const N: usize> DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
{
  const INSTANCE_CHECK: () = {
    assert!(N <= L::UPPER_BOUND_USIZE);
  };

  /// Constructs a new, empty instance.
  ///
  /// ```rust
  /// use wtx::collections::LinearStorageLen;
  /// let mut vec = wtx::collections::DynVectorU7::<u8, 16>::new();
  /// assert_eq!(vec.len().usize(), 0);
  /// ```
  #[inline]
  pub const fn new() -> Self {
    const { Self::INSTANCE_CHECK };
    Self(dyn_vector_inner::DynVectorInner::new())
  }

  /// Constructs a new, empty instance with at least the specified capacity.
  ///
  /// ```rust
  /// use wtx::collections::LinearStorageLen;
  /// let vec = wtx::collections::DynVectorU7::<u8, 16>::with_capacity(4).unwrap();
  /// assert!(vec.capacity().usize() >= 4);
  /// ```
  #[inline]
  pub fn with_capacity(capacity: L::Storage) -> crate::Result<Self> {
    const { Self::INSTANCE_CHECK };
    let mut this = Self::new();
    this.0.reserve_exact(L::from(capacity))?;
    Ok(this)
  }

  #[doc = from_cloneable_elem_doc!("DynVectorU7::<u8, 16>")]
  #[inline]
  pub fn from_cloneable_elem(len: L::Storage, value: T) -> crate::Result<Self>
  where
    T: Clone,
  {
    const { Self::INSTANCE_CHECK };
    Ok(Self(dyn_vector_inner::DynVectorInner::from_cloneable_elem(L::from(len), value)?))
  }

  #[doc = from_cloneable_slice_doc!("DynVectorU7::<u8, 16>")]
  #[inline]
  pub fn from_cloneable_slice(slice: &[T]) -> crate::Result<Self>
  where
    T: Clone,
  {
    const { Self::INSTANCE_CHECK };
    Ok(Self(dyn_vector_inner::DynVectorInner::from_cloneable_slice(slice)?))
  }

  #[doc = from_copyable_slice_doc!("DynVectorU7::<u8, 16>")]
  #[inline]
  pub fn from_copyable_slice(slice: &[T]) -> crate::Result<Self>
  where
    T: Copy,
  {
    const { Self::INSTANCE_CHECK };
    Ok(Self(dyn_vector_inner::DynVectorInner::from_copyable_slice(slice)?))
  }

  #[doc = from_iter_doc!("DynVectorU7::<u8, 16>", "[1, 2, 3]", "&[1, 2, 3]")]
  #[inline]
  pub fn from_iterator(iter: impl IntoIterator<Item = T>) -> crate::Result<Self> {
    const { Self::INSTANCE_CHECK };
    Ok(Self(dyn_vector_inner::DynVectorInner::from_iterator(iter)?))
  }

  #[doc = allocated!("DynVectorU7::<u8, 16>")]
  #[inline]
  pub fn allocated(&self) -> &[MaybeUninit<T>] {
    self.0.allocated()
  }

  #[doc = as_ptr_doc!("DynVectorU7::<u8, 16>")]
  #[inline]
  pub fn as_ptr(&self) -> *const T {
    self.0.as_ptr()
  }

  #[doc = as_ptr_mut_doc!("DynVectorU7::<u8, 16>")]
  #[inline]
  pub fn as_ptr_mut(&mut self) -> *mut T {
    self.0.as_ptr_mut()
  }

  #[doc = as_slice_doc!("DynVectorU7::<u8, 16>", "[1, 2, 3]", "[1, 2, 3]")]
  #[inline]
  pub fn as_slice(&self) -> &[T] {
    self.0.as_slice()
  }

  #[doc = as_slice_mut_doc!("DynVectorU7::<u8, 16>", "[1, 2, 3]", "&[1, 2, 3]")]
  #[inline]
  pub fn as_slice_mut(&mut self) -> &mut [T] {
    self.0.as_slice_mut()
  }

  #[doc = capacity_doc!("DynVectorU7::<u8, 16>", "[1, 2, 3]")]
  #[inline]
  pub fn capacity(&self) -> L {
    self.0.capacity()
  }

  #[doc = clear_doc!("DynVectorU7::<u8, 16>", "[1, 2, 3]")]
  #[inline]
  pub fn clear(&mut self) {
    self.0.clear();
  }

  #[doc = expand_doc!("DynVectorU7::<u8, 16>")]
  #[inline]
  pub fn expand(&mut self, et: ExpansionTy<L::Storage>, value: T) -> crate::Result<()>
  where
    T: Clone,
  {
    let converted_et = match et {
      ExpansionTy::Additional(elem) => ExpansionTy::Additional(L::from(elem)),
      ExpansionTy::Len(elem) => ExpansionTy::Len(L::from(elem)),
    };
    self.0.expand(converted_et, value)
  }

  #[doc = extend_from_cloneable_slice_doc!("DynVectorU7::<u8, 16>")]
  #[inline]
  pub fn extend_from_cloneable_slice(&mut self, other: &[T]) -> crate::Result<()>
  where
    T: Clone,
  {
    self.0.extend_from_cloneable_slice(other)
  }

  #[doc = extend_from_copyable_slice_doc!("DynVectorU7::<u8, 16>")]
  #[inline]
  pub fn extend_from_copyable_slice(&mut self, other: &[T]) -> crate::Result<()>
  where
    T: Copy,
  {
    self.0.extend_from_copyable_slice(other)
  }

  #[doc = extend_from_copyable_slice_doc!("DynVectorU7::<u8, 16>")]
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

  #[doc = extend_from_iter_doc!("DynVectorU7::<u8, 16>", "[1, 2, 3]", "&[1, 2, 3]")]
  #[inline]
  pub fn extend_from_iter(&mut self, iter: impl IntoIterator<Item = T>) -> crate::Result<()> {
    self.0.extend_from_iter(iter)
  }

  #[doc = insert_doc!("DynVectorU7::<u8, 16>")]
  #[inline]
  pub fn insert(&mut self, idx: L::Storage, elem: T) -> crate::Result<()> {
    self.0.insert(DynVectorError::OutOfBoundsInsertIdx.into(), L::from(idx), elem)
  }

  #[doc = len_doc!("DynVectorU7::<u8, 16>", "[1, 2, 3]")]
  #[inline]
  pub fn len(&self) -> L {
    self.0.len()
  }

  #[doc = pop_doc!("DynVectorU7::<u8, 16>", "[1, 2, 3]", "[1, 2]")]
  #[inline]
  pub fn pop(&mut self) -> Option<T> {
    <[T] as LinearStorageSlice>::pop(&mut self.0)
  }

  #[doc = push_doc!("DynVectorU7::<u8, 16>", "1", "&[1]")]
  #[inline]
  pub fn push(&mut self, elem: T) -> crate::Result<()> {
    self.0.push(elem)
  }

  #[doc = remaining_capacity_doc!("DynVectorU7::<u8, 16>", "1")]
  #[inline]
  pub fn remaining_capacity(&self) -> L {
    self.0.remaining_capacity()
  }

  #[doc = remaining_capacity_max_doc!("DynVectorU7::<u8, 16>", "[1, 2, 3]", "wtx::collections::U7Len::MAX.get() as usize - 3")]
  #[inline]
  pub fn remaining_capacity_max(&self) -> L {
    self.0.remaining_capacity_max()
  }

  #[doc = remove_doc!("DynVectorU7::<u8, 16>", "[1, 2, 3]", "[1, 3]")]
  #[inline]
  pub fn remove(&mut self, index: L::Storage) -> Option<T> {
    <[T] as LinearStorageSlice>::remove(&mut self.0, L::from(index))
  }

  #[doc = reserve_doc!("DynVectorU7::<u8, 16>")]
  #[inline]
  pub fn reserve(&mut self, additional: L::Storage) -> crate::Result<()> {
    self.0.reserve(L::from(additional))
  }

  #[doc = reserve_exact_doc!("DynVectorU7::<u8, 16>")]
  #[inline]
  pub fn reserve_exact(&mut self, additional: L::Storage) -> crate::Result<()> {
    self.0.reserve_exact(L::from(additional))
  }

  #[doc = retain_unstable_doc!("DynVectorU7::<u8, 16>")]
  #[inline]
  pub fn retain_unstable(&mut self, cb: impl FnMut(&T) -> bool) {
    <[T] as LinearStorageSlice>::retain_unstable(&mut self.0, cb);
  }

  #[doc = set_len_doc!("DynVectorU7::<u8, 16>", "[1, 2, 3]", "&[1]")]
  #[inline]
  pub unsafe fn set_len(&mut self, new_len: L::Storage) {
    // SAFETY: Up to the caller
    unsafe {
      self.0.set_len(L::from(new_len));
    }
  }

  #[doc = split_at_spare_mut!("DynVectorU7::<u8, 16>")]
  #[inline]
  pub fn split_at_spare_mut(&mut self) -> (&mut [T], &mut [MaybeUninit<T>]) {
    self.0.split_at_spare_mut()
  }

  #[doc = swap_remove_doc!("DynVectorU7::<u8, 16>")]
  #[inline]
  pub fn swap_remove(&mut self, index: L::Storage) -> Option<T> {
    <[T] as LinearStorageSlice>::swap_remove(&mut self.0, L::from(index))
  }

  #[doc = truncate_doc!("DynVectorU7::<u8, 16>", "[1, 2, 3]", "[1]")]
  #[inline]
  pub fn truncate(&mut self, new_len: L::Storage) {
    let _rslt = <[T] as LinearStorageSlice>::truncate(&mut self.0, L::from(new_len));
  }
}

impl<L, T, const N: usize> Clone for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
  T: Clone,
{
  #[inline]
  fn clone(&self) -> Self {
    let mut this = Self::new();
    if this.0.extend_from_cloneable_slice(self).is_err() {
      unlikely_unreachable();
    }
    this
  }
}

impl<L, T, const N: usize> Debug for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
  T: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    self.lease().fmt(f)
  }
}

impl<L, T, const N: usize> Default for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<L, T, const N: usize> Deref for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
{
  type Target = [T];

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.as_slice()
  }
}

impl<L, T, const N: usize> DerefMut for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.as_slice_mut()
  }
}

impl<L, T, const N: usize> Eq for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
  T: Eq,
{
}

impl<L, T, const N: usize> Hash for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
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

impl<'any, L, T, const N: usize> IntoIterator for &'any DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
  T: 'any,
{
  type IntoIter = slice::Iter<'any, T>;
  type Item = &'any T;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

impl<'any, L, T, const N: usize> IntoIterator for &'any mut DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
  T: 'any,
{
  type IntoIter = slice::IterMut<'any, T>;
  type Item = &'any mut T;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.iter_mut()
  }
}

impl<L, T, const N: usize> Lease<[T]> for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn lease(&self) -> &[T] {
    self
  }
}

impl<L, T, const N: usize> LeaseMut<[T]> for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn lease_mut(&mut self) -> &mut [T] {
    self
  }
}

impl<L, T, const N: usize> PartialEq for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
  T: PartialEq,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

impl<L, T, U, const N: usize> PartialEq<[U]> for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &[U]) -> bool {
    **self == *other
  }
}
impl<L, T, U, const N: usize> PartialEq<&[U]> for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&[U]) -> bool {
    **self == **other
  }
}
impl<L, T, U, const N: usize> PartialEq<&mut [U]> for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&mut [U]) -> bool {
    **self == **other
  }
}

impl<L, T, U, const M: usize, const N: usize> PartialEq<[U; M]> for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &[U; M]) -> bool {
    **self == *other
  }
}
impl<L, T, U, const M: usize, const N: usize> PartialEq<&[U; M]> for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&[U; M]) -> bool {
    **self == **other
  }
}
impl<L, T, U, const M: usize, const N: usize> PartialEq<&mut [U; M]> for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&mut [U; M]) -> bool {
    **self == **other
  }
}

impl<L, T, const N: usize> PartialOrd for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
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

impl<L, T, const N: usize> Ord for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
  T: Ord,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    (**self).cmp(&**other)
  }
}

impl<L, T, const N: usize> Truncate for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn truncate(&mut self, input: usize) {
    let new_len = L::from_usize(input).unwrap_or(L::UPPER_BOUND);
    let _rslt = <[T] as LinearStorageSlice>::truncate(&mut self.0, new_len);
  }
}

impl<'args, L, const N: usize> TryFrom<Arguments<'args>> for DynVector<L, u8, N>
where
  L: SparseLinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(from: Arguments<'args>) -> Result<Self, Self::Error> {
    let mut rslt = Self::new();
    fmt::Write::write_fmt(&mut rslt, from)?;
    Ok(rslt)
  }
}

impl<L, T, const N: usize> TryFrom<&[T]> for DynVector<L, T, N>
where
  L: SparseLinearStorageLen,
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

impl<L, const N: usize> fmt::Write for DynVector<L, u8, N>
where
  L: SparseLinearStorageLen,
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
impl<L, const N: usize> std::io::Write for DynVector<L, u8, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }

  #[inline]
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    let len = (self.0.remaining_capacity().usize()).min(buf.len());
    let _rslt = self.0.extend_from_copyable_slice(buf.get(..len).unwrap_or_default());
    Ok(len)
  }
}
