use crate::{
  collection::{
    LinearStorageLen,
    linear_storage::{LinearStorage, linear_storage_mut::LinearStorageMut},
  },
  misc::{Lease, LeaseMut},
};
use core::{
  cmp::Ordering,
  fmt::{self, Debug, Formatter},
  hash::{Hash, Hasher},
  mem::MaybeUninit,
  ops::{Deref, DerefMut},
  slice,
};

/// [`Uninit`] with a capacity limited by `u8`.
pub type UninitU8<'data, T> = Uninit<'data, u8, T>;
/// [`Uninit`] with a capacity limited by `u16`.
pub type UninitU16<'data, T> = Uninit<'data, u16, T>;
/// [`Uninit`] with a capacity limited by `u32`.
pub type UninitU32<'data, T> = Uninit<'data, u32, T>;
/// [`Uninit`] with a capacity limited by `usize`.
pub type UninitUsize<'data, T> = Uninit<'data, usize, T>;

/// Errors of [`Uninit`].
#[derive(Debug)]
pub enum UninitError {
  #[doc = doc_reserve_overflow!()]
  ReserveOverflow,
}

/// Uninitialized slice
pub struct Uninit<'data, L, T>(Inner<'data, L, T>)
where
  L: LinearStorageLen;

impl<'data, L, T> Uninit<'data, L, T>
where
  L: LinearStorageLen,
{
  /// Constructs a new instance.
  #[inline]
  pub const fn new(data: &'data mut [MaybeUninit<T>]) -> Self {
    Self(Inner { len: L::ZERO, data })
  }
}

impl<L, T> Uninit<'_, L, T>
where
  L: LinearStorageLen,
{
  /// Returns the initialized elements
  #[inline]
  pub fn as_slice(&self) -> &[T] {
    self.0.as_slice()
  }

  #[doc = as_slice_mut_doc!()]
  #[inline]
  pub fn as_slice_mut(&mut self) -> &mut [T] {
    self.0.as_slice_mut()
  }

  /// Alias for the length of the slice
  #[inline]
  pub fn capacity(&self) -> L {
    self.0.capacity()
  }

  /// Resets state as if no element was written.
  #[inline]
  pub fn clear(&mut self) {
    self.0.len = L::ZERO;
  }

  #[doc = extend_from_cloneable_slice_doc!("UninitUsize", ("slice", "&mut [core::mem::MaybeUninit::uninit(); 16]"))]
  #[inline]
  pub fn extend_from_cloneable_slice(&mut self, other: &[T]) -> crate::Result<()>
  where
    T: Clone,
  {
    self.0.extend_from_cloneable_slice(other)
  }

  #[doc = extend_from_copyable_slice_doc!("UninitUsize", ("slice", "&mut [core::mem::MaybeUninit::uninit(); 16]"))]
  #[inline]
  pub fn extend_from_copyable_slice(&mut self, other: &[T]) -> crate::Result<()>
  where
    T: Copy,
  {
    self.0.extend_from_copyable_slice(other)
  }

  #[doc = extend_from_copyable_slice_doc!("UninitUsize", ("slice", "&mut [core::mem::MaybeUninit::uninit(); 16]"))]
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

  #[doc = extend_from_iter_doc!("UninitUsize", "[1, 2, 3]", "&[1, 2, 3]", ("slice", "&mut [core::mem::MaybeUninit::uninit(); 16]"))]
  #[inline]
  pub fn extend_from_iter(&mut self, iter: impl IntoIterator<Item = T>) -> crate::Result<()> {
    self.0.extend_from_iter(iter)
  }

  #[doc = len_doc!()]
  #[inline]
  pub fn len(&self) -> L {
    self.0.len()
  }

  #[doc = push_doc!("UninitUsize", "1", "&[1]", ("slice", "&mut [core::mem::MaybeUninit::uninit(); 16]"))]
  #[inline]
  pub fn push(&mut self, elem: T) -> crate::Result<()> {
    self.0.push(elem)
  }
}

impl<L, T> Debug for Uninit<'_, L, T>
where
  L: LinearStorageLen,
  T: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    self.lease().fmt(f)
  }
}

impl<L, T> Deref for Uninit<'_, L, T>
where
  L: LinearStorageLen,
{
  type Target = [T];

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.0.as_slice()
  }
}

impl<L, T> DerefMut for Uninit<'_, L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.0.as_slice_mut()
  }
}

impl<L, T> Drop for Uninit<'_, L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn drop(&mut self) {
    if Inner::<'_, L, T>::NEEDS_DROP {
      self.0.clear();
    }
  }
}

impl<L, T> Eq for Uninit<'_, L, T>
where
  L: LinearStorageLen,
  T: Eq,
{
}

impl<L, T> Hash for Uninit<'_, L, T>
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

impl<'any, L, T> IntoIterator for &'any Uninit<'_, L, T>
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

impl<'any, L, T> IntoIterator for &'any mut Uninit<'_, L, T>
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

impl<L, T> Lease<[T]> for Uninit<'_, L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn lease(&self) -> &[T] {
    self
  }
}

impl<L, T> LeaseMut<[T]> for Uninit<'_, L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn lease_mut(&mut self) -> &mut [T] {
    self
  }
}

impl<L, T> PartialEq for Uninit<'_, L, T>
where
  L: LinearStorageLen,
  T: PartialEq,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

impl<L, T, U> PartialEq<[U]> for Uninit<'_, L, T>
where
  L: LinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &[U]) -> bool {
    **self == *other
  }
}
impl<L, T, U> PartialEq<&[U]> for Uninit<'_, L, T>
where
  L: LinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&[U]) -> bool {
    **self == **other
  }
}
impl<L, T, U> PartialEq<&mut [U]> for Uninit<'_, L, T>
where
  L: LinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&mut [U]) -> bool {
    **self == **other
  }
}

impl<L, T, U, const M: usize> PartialEq<[U; M]> for Uninit<'_, L, T>
where
  L: LinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &[U; M]) -> bool {
    **self == *other
  }
}
impl<L, T, U, const M: usize> PartialEq<&[U; M]> for Uninit<'_, L, T>
where
  L: LinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&[U; M]) -> bool {
    **self == **other
  }
}
impl<L, T, U, const M: usize> PartialEq<&mut [U; M]> for Uninit<'_, L, T>
where
  L: LinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&mut [U; M]) -> bool {
    **self == **other
  }
}

impl<L, T> PartialOrd for Uninit<'_, L, T>
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

impl<L, T> Ord for Uninit<'_, L, T>
where
  L: LinearStorageLen,
  T: Ord,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    (**self).cmp(&**other)
  }
}

struct Inner<'data, L, T>
where
  L: LinearStorageLen,
{
  len: L,
  data: &'data mut [MaybeUninit<T>],
}

impl<L, T> LinearStorage<T> for Inner<'_, L, T>
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
    L::from_usize(self.data.len()).unwrap_or(L::UPPER_BOUND)
  }

  #[inline]
  fn len(&self) -> Self::Len {
    self.len
  }
}

impl<L, T> LinearStorageMut<T> for Inner<'_, L, T>
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
      return Err(UninitError::ReserveOverflow.into());
    }
    Ok(())
  }

  #[inline]
  fn reserve_exact(&mut self, additional: Self::Len) -> crate::Result<()> {
    if additional > self.remaining() {
      return Err(UninitError::ReserveOverflow.into());
    }
    Ok(())
  }

  #[inline]
  unsafe fn set_len(&mut self, new_len: Self::Len) {
    self.len = new_len;
  }
}
