use crate::{
  collections::{
    ExpansionTy, LinearStorageLen, ShortBoxSlice, SuffixGuardVectorMut,
    linear_storage::{
      LinearStorage, linear_storage_mut::LinearStorageMut, linear_storage_slice::LinearStorageSlice,
    },
  },
  misc::{Lease, LeaseMut, Wrapper, unlikely_unreachable},
};
use alloc::{
  string::String,
  vec::{IntoIter, Vec},
};
use core::{
  borrow::{Borrow, BorrowMut},
  cmp::Ordering,
  fmt::{Debug, Display, Formatter},
  hash::{Hash, Hasher},
  marker::PhantomData,
  mem::{self, ManuallyDrop, MaybeUninit},
  ops::{Deref, DerefMut},
  ptr::NonNull,
  slice::{Iter, IterMut},
};

/// [`Vector`] with a capacity limited by `u8`.
pub type VectorU8<T> = Vector<u8, T>;
/// [`Vector`] with a capacity limited by `u16`.
pub type VectorU16<T> = Vector<u16, T>;
/// [`Vector`] with a capacity limited by `u32`.
pub type VectorU32<T> = Vector<u32, T>;
/// [`Vector`] with a capacity limited by `usize`.
pub type VectorUsize<T> = Vector<usize, T>;

/// Errors of [Vector].
#[derive(Clone, Copy, Debug)]
pub enum VectorError {
  #[doc = doc_reserve_overflow!()]
  CapacityOverflow,
  #[doc = doc_many_elems_cap_overflow!()]
  ExtendFromSliceOverflow,
  #[doc = doc_many_elems_cap_overflow!()]
  ExtendFromSlicesOverflow,
  /// When converting to the vector of the standard library, internal parameters were transformed
  /// into overflowing values.
  InvalidStdConversion,
  /// The index provided in the `insert` method is out of bounds.
  OutOfBoundsInsertIdx,
  #[doc = doc_single_elem_cap_overflow!()]
  PushOverflow,
  #[doc = doc_reserve_overflow!()]
  ReserveOverflow {
    /// Additional
    additional: u16,
    /// Current
    curr: u32,
    /// Maximum
    max: u32,
  },
  /// A temporary `Vec` expanded the capacity beyond the current length type
  VecOverflow,
}

impl Display for VectorError {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    <Self as Debug>::fmt(self, f)
  }
}

impl From<VectorError> for u8 {
  #[inline]
  fn from(from: VectorError) -> Self {
    match from {
      VectorError::CapacityOverflow => 0,
      VectorError::ExtendFromSliceOverflow => 1,
      VectorError::ExtendFromSlicesOverflow => 2,
      VectorError::InvalidStdConversion => 3,
      VectorError::OutOfBoundsInsertIdx => 4,
      VectorError::PushOverflow => 5,
      VectorError::ReserveOverflow { additional: _, curr: _, max: _ } => 6,
      VectorError::VecOverflow => 7,
    }
  }
}

impl core::error::Error for VectorError {}

/// A wrapper around the std's vector.
pub struct Vector<L, T>(Inner<L, T>)
where
  L: LinearStorageLen;

impl<L, T> Vector<L, T>
where
  L: LinearStorageLen,
{
  /// Constructs a new instance based on an arbitrary [Vec].
  ///
  /// ```rust
  /// let mut vec = wtx::collections::VectorU8::<u8>::from_vec(Vec::new()).unwrap();
  /// assert_eq!(vec.len(), 0);
  /// ```
  #[inline]
  pub fn from_vec(vec: Vec<T>) -> crate::Result<Self> {
    Ok(Self(Inner::from_vec(vec)?))
  }

  /// Constructs a new, empty instance.
  ///
  /// ```rust
  /// let mut vec = wtx::collections::VectorU8::<u8>::new();
  /// assert_eq!(vec.len(), 0);
  /// ```
  #[inline]
  pub const fn new() -> Self {
    Self(Inner::new())
  }

  /// Constructs a new, empty instance with at least the specified capacity.
  /// Constructs a new instance based on an arbitrary [Vec].
  ///
  /// ```rust
  /// let mut vec = wtx::collections::VectorU8::<u8>::with_capacity(2).unwrap();
  /// assert!(vec.capacity() >= 2);
  /// ```
  #[inline(always)]
  pub fn with_capacity(capacity: L) -> crate::Result<Self> {
    Ok(Self(Inner::with_capacity(capacity)?))
  }

  /// Constructs a new, empty instance with the exact specified capacity.
  ///
  /// ```rust
  /// let mut vec = wtx::collections::VectorU8::<u8>::with_exact_capacity(2).unwrap();
  /// assert_eq!(vec.capacity(), 2);
  /// ```
  #[inline(always)]
  pub fn with_exact_capacity(capacity: L) -> crate::Result<Self> {
    let mut this = Self::new();
    this.reserve_exact(capacity)?;
    Ok(this)
  }

  /// Transfers memory ownership to the vector of the standard library.
  ///
  /// ```rust
  /// let vec = wtx::collections::VectorU8::<u8>::new();
  /// assert_eq!(vec.into_vec(), Vec::<u8>::new());
  /// ```
  #[inline]
  pub fn into_vec(self) -> Vec<T> {
    self.0.into_vec()
  }

  /// Vector of the standard library.
  #[inline]
  pub fn to_vec_mut<U>(
    &mut self,
    cb: impl FnOnce(&mut Vec<T>) -> crate::Result<U>,
  ) -> crate::Result<U> {
    self.0.to_vec_mut(cb)
  }
}

impl<L, T> Vector<L, T>
where
  L: LinearStorageLen,
{
  #[doc = from_cloneable_elem_doc!("VectorU8")]
  #[inline]
  pub fn from_cloneable_elem(len: usize, value: T) -> crate::Result<Self>
  where
    T: Clone,
  {
    Ok(Self(Inner::from_cloneable_elem(len, value)?))
  }

  #[doc = from_cloneable_slice_doc!("VectorU8")]
  #[inline]
  pub fn from_cloneable_slice(slice: &[T]) -> crate::Result<Self>
  where
    T: Clone,
  {
    Ok(Self(Inner::from_cloneable_slice(slice)?))
  }

  #[doc = from_copyable_slice_doc!("VectorU8")]
  #[inline]
  pub fn from_copyable_slice(slice: &[T]) -> crate::Result<Self>
  where
    T: Copy,
  {
    Ok(Self(Inner::from_copyable_slice(slice)?))
  }

  #[doc = from_iter_doc!("VectorU8", "[1, 2, 3]", "&[1, 2, 3]")]
  #[inline]
  pub fn from_iterator(iter: impl IntoIterator<Item = T>) -> crate::Result<Self> {
    Ok(Self(Inner::from_iterator(iter)?))
  }

  #[doc = allocated!("VectorU8::<u8>")]
  #[inline]
  pub fn allocated(&self) -> &[MaybeUninit<T>] {
    self.0.allocated()
  }

  #[doc = as_ptr_doc!("VectorUsize", "[1, 2, 3]")]
  #[inline]
  pub fn as_ptr(&self) -> *const T {
    self.0.as_ptr()
  }

  #[doc = as_ptr_mut_doc!()]
  #[inline]
  pub fn as_ptr_mut(&mut self) -> *mut T {
    self.0.as_ptr_mut()
  }

  #[doc = as_slice_doc!("VectorU8", "[1, 2, 3]", "[1, 2, 3]")]
  #[inline]
  pub fn as_slice(&self) -> &[T] {
    self.0.as_slice()
  }

  #[doc = as_slice_mut_doc!()]
  #[inline]
  pub fn as_slice_mut(&mut self) -> &mut [T] {
    self.0.as_slice_mut()
  }

  #[doc = capacity_doc!("VectorU8", "[1, 2, 3]")]
  #[inline]
  pub fn capacity(&self) -> L {
    self.0.capacity()
  }

  #[doc = clear_doc!("VectorU8", "[1, 2, 3]")]
  #[inline]
  pub fn clear(&mut self) {
    self.0.clear();
  }

  #[doc = expand_doc!("VectorU8")]
  #[inline]
  pub fn expand(&mut self, et: ExpansionTy, value: T) -> crate::Result<()>
  where
    T: Clone,
  {
    self.0.expand(et, value)
  }

  #[doc = extend_from_cloneable_slice_doc!("VectorU8")]
  #[inline]
  pub fn extend_from_cloneable_slice(&mut self, other: &[T]) -> crate::Result<()>
  where
    T: Clone,
  {
    self.0.extend_from_cloneable_slice(other)
  }

  #[doc = extend_from_copyable_slice_doc!("VectorU8")]
  #[inline]
  pub fn extend_from_copyable_slice(&mut self, other: &[T]) -> crate::Result<()>
  where
    T: Copy,
  {
    self.0.extend_from_copyable_slice(other)
  }

  #[doc = extend_from_copyable_slice_doc!("VectorU8")]
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

  #[doc = extend_from_iter_doc!("VectorU8", "[1, 2, 3]", "&[1, 2, 3]")]
  #[inline]
  pub fn extend_from_iter(&mut self, iter: impl IntoIterator<Item = T>) -> crate::Result<()> {
    self.0.extend_from_iter(iter)
  }

  #[doc = insert_doc!("VectorU8")]
  #[inline]
  pub fn insert(&mut self, idx: L, elem: T) -> crate::Result<()> {
    self.0.insert(VectorError::OutOfBoundsInsertIdx.into(), idx, elem)
  }

  #[doc = len_doc!()]
  #[inline]
  pub fn len(&self) -> L {
    self.0.len()
  }

  #[doc = pop_doc!("VectorU8", "[1, 2, 3]", "[1, 2]")]
  #[inline]
  pub fn pop(&mut self) -> Option<T> {
    <[T] as LinearStorageSlice>::pop(&mut self.0)
  }

  #[doc = push_doc!("VectorU8", "1", "&[1]")]
  #[inline]
  pub fn push(&mut self, elem: T) -> crate::Result<()> {
    self.0.push(elem)
  }

  #[doc = remaining_capacity_doc!("VectorU8", "1")]
  #[inline]
  pub fn remaining_capacity(&self) -> L {
    self.0.remaining_capacity()
  }

  #[doc = remaining_capacity_max_doc!("VectorU8")]
  #[inline]
  pub fn remaining_capacity_max(&self) -> L {
    self.0.remaining_capacity_max()
  }

  #[doc = remove_doc!("VectorU8", "[1, 2, 3]", "[1, 3]")]
  #[inline]
  pub fn remove(&mut self, index: L) -> Option<T> {
    <[T] as LinearStorageSlice>::remove(&mut self.0, index)
  }

  #[doc = reserve_doc!("VectorU8::<u8>")]
  #[inline]
  pub fn reserve(&mut self, additional: L) -> crate::Result<()> {
    self.0.reserve(additional)
  }

  #[doc = reserve_exact_doc!("VectorU8::<u8>")]
  #[inline]
  pub fn reserve_exact(&mut self, additional: L) -> crate::Result<()> {
    self.0.reserve_exact(additional)
  }

  #[doc = set_len_doc!()]
  #[inline]
  pub unsafe fn set_len(&mut self, new_len: L) {
    // SAFETY: Up to the caller
    unsafe {
      self.0.set_len(new_len);
    }
  }

  #[doc = split_at_spare_mut!("VectorU8")]
  #[inline]
  pub fn split_at_spare_mut(&mut self) -> (&mut [T], &mut [MaybeUninit<T>]) {
    self.0.split_at_spare_mut()
  }

  /// See [`SuffixGuardVectorMut`].
  #[inline]
  pub fn suffix_pusher(&mut self) -> SuffixGuardVectorMut<'_, L, T> {
    SuffixGuardVectorMut::from(self)
  }

  #[doc = swap_remove_doc!("VectorU8")]
  #[inline]
  pub fn swap_remove(&mut self, index: L) -> Option<T> {
    <[T] as LinearStorageSlice>::swap_remove(&mut self.0, index)
  }

  #[doc = truncate_doc!("VectorU8", "[1, 2, 3]", "[1]")]
  #[inline]
  pub fn truncate(&mut self, new_len: L) {
    let _rslt = <[T] as LinearStorageSlice>::truncate(&mut self.0, new_len);
  }
}

impl<L, T> Lease<[T]> for Vector<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn lease(&self) -> &[T] {
    self
  }
}

impl<L, T> Lease<Vector<L, T>> for Vector<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn lease(&self) -> &Vector<L, T> {
    self
  }
}

impl<L, T> LeaseMut<[T]> for Vector<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn lease_mut(&mut self) -> &mut [T] {
    self
  }
}

impl<L, T> LeaseMut<Vector<L, T>> for Vector<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn lease_mut(&mut self) -> &mut Vector<L, T> {
    self
  }
}

impl<L, T> AsMut<[T]> for Vector<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn as_mut(&mut self) -> &mut [T] {
    self
  }
}

impl<L, T> AsRef<[T]> for Vector<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn as_ref(&self) -> &[T] {
    self
  }
}

impl<L, T> Borrow<[T]> for Vector<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn borrow(&self) -> &[T] {
    self
  }
}

impl<L, T> BorrowMut<[T]> for Vector<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn borrow_mut(&mut self) -> &mut [T] {
    self
  }
}

impl<L, T> Clone for Vector<L, T>
where
  L: LinearStorageLen,
  T: Clone,
{
  #[inline]
  #[track_caller]
  fn clone(&self) -> Self {
    let Ok(mut vector) = Self::with_capacity(self.len()) else {
      unlikely_unreachable();
    };
    let _rslt = vector.extend_from_cloneable_slice(self);
    vector
  }

  #[inline]
  fn clone_from(&mut self, source: &Self) {
    self.truncate(source.len());
    let (init, tail) = source.split_at(self.len().usize());
    self.clone_from_slice(init);
    if self.extend_from_cloneable_slice(tail).is_err() {
      unlikely_unreachable();
    }
  }
}

impl<L, T> Debug for Vector<L, T>
where
  L: LinearStorageLen,
  T: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    self.0.as_slice().fmt(f)
  }
}

impl<L, T> Default for Vector<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<L, T> Deref for Vector<L, T>
where
  L: LinearStorageLen,
{
  type Target = [T];

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.0.as_slice()
  }
}

impl<L, T> DerefMut for Vector<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.0.as_slice_mut()
  }
}

impl<L, T> From<Vector<L, T>> for Vec<T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn from(from: Vector<L, T>) -> Self {
    from.0.into_vec()
  }
}

impl<L> TryFrom<String> for Vector<L, u8>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: String) -> Result<Self, Self::Error> {
    Vector::from_vec(value.into())
  }
}

impl<L> TryFrom<Vector<L, u8>> for String
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: Vector<L, u8>) -> Result<Self, Self::Error> {
    Ok(String::from_utf8(Vec::<u8>::from(value))?)
  }
}

impl<L0, L1, T> From<ShortBoxSlice<L0, T>> for Vector<L1, T>
where
  L0: LinearStorageLen,
  L1: LinearStorageLen,
{
  #[inline]
  fn from(value: ShortBoxSlice<L0, T>) -> Self {
    Vector::from_vec(Vec::<T>::from(value)).unwrap_or_default()
  }
}

impl<L, T> FromIterator<T> for Wrapper<crate::Result<Vector<L, T>>>
where
  L: LinearStorageLen,
{
  #[inline]
  fn from_iter<I>(iter: I) -> Self
  where
    I: IntoIterator<Item = T>,
  {
    Wrapper(Vector::from_iterator(iter))
  }
}

impl<L, T> Eq for Vector<L, T>
where
  L: LinearStorageLen,
  T: Eq,
{
}

impl<L, T> Hash for Vector<L, T>
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

impl<L, T> IntoIterator for Vector<L, T>
where
  L: LinearStorageLen,
{
  type Item = T;
  type IntoIter = IntoIter<T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.into_vec().into_iter()
  }
}

impl<'any, L, T> IntoIterator for &'any Vector<L, T>
where
  L: LinearStorageLen,
{
  type Item = &'any T;
  type IntoIter = Iter<'any, T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

impl<'any, L, T> IntoIterator for &'any mut Vector<L, T>
where
  L: LinearStorageLen,
{
  type Item = &'any mut T;
  type IntoIter = IterMut<'any, T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.iter_mut()
  }
}

impl<L, T> Ord for Vector<L, T>
where
  L: LinearStorageLen,
  T: Ord,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    (**self).cmp(&**other)
  }
}

impl<L, T> PartialEq for Vector<L, T>
where
  L: LinearStorageLen,
  T: PartialEq,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

impl<L, T, U> PartialEq<[U]> for Vector<L, T>
where
  L: LinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &[U]) -> bool {
    **self == *other
  }
}

impl<L, T, U> PartialEq<&[U]> for Vector<L, T>
where
  L: LinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&[U]) -> bool {
    **self == **other
  }
}

impl<L, T, U> PartialEq<&mut [U]> for Vector<L, T>
where
  L: LinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&mut [U]) -> bool {
    **self == **other
  }
}

impl<L, T, U, const N: usize> PartialEq<[U; N]> for Vector<L, T>
where
  L: LinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &[U; N]) -> bool {
    **self == *other
  }
}

impl<L, T, U, const N: usize> PartialEq<&[U; N]> for Vector<L, T>
where
  L: LinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&[U; N]) -> bool {
    **self == **other
  }
}

impl<L, T, U, const N: usize> PartialEq<&mut [U; N]> for Vector<L, T>
where
  L: LinearStorageLen,
  T: PartialEq<U>,
{
  #[inline]
  fn eq(&self, other: &&mut [U; N]) -> bool {
    **self == **other
  }
}

impl<L, T> PartialOrd for Vector<L, T>
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

impl<L, T> TryFrom<Vec<T>> for Vector<L, T>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
    Self::from_vec(value)
  }
}

impl<L, T> TryFrom<&[T]> for Vector<L, T>
where
  L: LinearStorageLen,
  T: Clone,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &[T]) -> Result<Self, Self::Error> {
    Self::from_cloneable_slice(value)
  }
}

impl<L> core::fmt::Write for Vector<L, u8>
where
  L: LinearStorageLen,
{
  #[inline]
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    self.extend_from_copyable_slice(s.as_bytes()).map_err(|_err| core::fmt::Error)
  }
}

#[cfg(feature = "std")]
impl<L> std::io::Write for Vector<L, u8>
where
  L: LinearStorageLen,
{
  #[inline]
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    self
      .extend_from_copyable_slice(buf)
      .map_err(|err| std::io::Error::new(std::io::ErrorKind::StorageFull, err))?;
    Ok(buf.len())
  }

  #[inline]
  fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
    Ok(
      self
        .extend_from_copyable_slices(bufs)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::StorageFull, err))?
        .usize(),
    )
  }

  #[inline]
  fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
    self
      .extend_from_copyable_slice(buf)
      .map_err(|err| std::io::Error::new(std::io::ErrorKind::StorageFull, err))?;
    Ok(())
  }

  #[inline]
  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }
}

#[expect(clippy::repr_packed_without_abi, reason = "not intended for FFI")]
#[repr(packed)]
struct Inner<L, T>
where
  L: LinearStorageLen,
{
  ptr: NonNull<u8>,
  cap: L,
  len: L,
  phantom: PhantomData<T>,
}

impl<L, T> Inner<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn with_capacity(capacity: L) -> crate::Result<Self> {
    Inner::from_vec(Vec::with_capacity(capacity.usize()))
  }

  #[inline]
  fn from_vec(mut vec: Vec<T>) -> crate::Result<Self> {
    if L::from_usize(vec.capacity()).is_err() {
      vec = vec.into_boxed_slice().into_vec();
    }
    let cap = L::from_usize(vec.capacity()).map_err(|_err| VectorError::CapacityOverflow)?;
    let len = L::from_usize(vec.len()).map_err(|_err| VectorError::CapacityOverflow)?;
    let (ptr, _, _) = vec.into_parts();
    Ok(Self { ptr: ptr.cast(), cap, len, phantom: PhantomData })
  }

  #[inline]
  const fn new() -> Self {
    Self {
      ptr: NonNull::<T>::dangling().cast::<u8>(),
      cap: L::ZERO,
      len: L::ZERO,
      phantom: PhantomData,
    }
  }

  #[inline]
  fn into_vec(self) -> Vec<T> {
    let this = ManuallyDrop::new(self);
    // SAFETY: Inner parameters are always valid
    unsafe { Vec::from_parts(this.ptr.cast(), this.len.usize(), this.cap.usize()) }
  }

  #[inline]
  fn to_vec_mut<U>(
    &mut self,
    cb: impl FnOnce(&mut Vec<T>) -> crate::Result<U>,
  ) -> crate::Result<U> {
    struct Guard<'any, L: LinearStorageLen, T> {
      is_ok: &'any mut bool,
      target: &'any mut Inner<L, T>,
      tmp_vec: Vec<T>,
    }
    impl<L, T> Drop for Guard<'_, L, T>
    where
      L: LinearStorageLen,
    {
      #[inline]
      fn drop(&mut self) {
        let tmp_vec_len = self.tmp_vec.len();
        let tmp_vec_cap = self.tmp_vec.capacity();
        if let (Ok(len_l), Ok(cap_l)) = (L::from_usize(tmp_vec_len), L::from_usize(tmp_vec_cap)) {
          let (ptr, _, _) = mem::take(&mut self.tmp_vec).into_parts();
          self.target.cap = cap_l;
          self.target.len = len_l;
          self.target.ptr = ptr.cast();
          *self.is_ok = true;
        }
      }
    }

    let cap = self.cap;
    let len = self.len;
    let ptr = self.ptr;
    self.cap = L::ZERO;
    self.len = L::ZERO;
    self.ptr = NonNull::<T>::dangling().cast::<u8>();
    // SAFETY: Inner parameters are always valid
    let tmp_vec = unsafe { Vec::from_parts(ptr.cast(), len.usize(), cap.usize()) };
    let mut is_ok = false;
    let rslt = {
      let mut guard = Guard { is_ok: &mut is_ok, target: self, tmp_vec };
      cb(&mut guard.tmp_vec)
    };
    if !is_ok {
      return Err(VectorError::InvalidStdConversion.into());
    }
    rslt
  }
}

impl<L, T> LinearStorage<T> for Inner<L, T>
where
  L: LinearStorageLen,
{
  type Len = L;
  type Slice = [T];

  #[inline]
  fn as_ptr(&self) -> *const T {
    self.ptr.as_ptr().cast()
  }

  #[inline]
  fn capacity(&self) -> Self::Len {
    self.cap
  }

  #[inline]
  fn len(&self) -> Self::Len {
    self.len
  }
}

impl<L, T> LinearStorageMut<T> for Inner<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn as_ptr_mut(&mut self) -> *mut T {
    self.ptr.as_ptr().cast()
  }

  #[inline]
  fn reserve(&mut self, additional: Self::Len) -> crate::Result<()> {
    let additional_usize = additional.usize();
    let len_usize = self.len().usize();
    self.to_vec_mut(|vec| {
      vec.try_reserve(additional_usize).map_err(|_err| VectorError::ReserveOverflow {
        additional: additional_usize.try_into().unwrap_or(u16::MAX),
        curr: len_usize.try_into().unwrap_or(u32::MAX),
        max: L::UPPER_BOUND_USIZE.try_into().unwrap_or(u32::MAX),
      })?;
      Ok(())
    })
  }

  #[inline]
  fn reserve_exact(&mut self, additional: Self::Len) -> crate::Result<()> {
    let additional_usize = additional.usize();
    let len_usize = self.len().usize();
    self.to_vec_mut(|vec| {
      vec.try_reserve_exact(additional_usize).map_err(|_err| VectorError::ReserveOverflow {
        additional: additional_usize.try_into().unwrap_or(u16::MAX),
        curr: len_usize.try_into().unwrap_or(u32::MAX),
        max: L::UPPER_BOUND_USIZE.try_into().unwrap_or(u32::MAX),
      })?;
      Ok(())
    })
  }

  #[inline]
  unsafe fn set_len(&mut self, new_len: Self::Len) {
    self.len = new_len;
  }
}

impl<L, T> Default for Inner<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<L, T> Drop for Inner<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn drop(&mut self) {
    let mut this = Vec::new();
    let _rslt = self.to_vec_mut(|el| {
      this = mem::take(el);
      Ok(())
    });
  }
}

// SAFETY: there is no immutable method internally operating mutable modifications
unsafe impl<L, T: Send> Send for Inner<L, T> where L: LinearStorageLen {}
// SAFETY: there is no immutable method internally operating mutable modifications
unsafe impl<L, T: Sync> Sync for Inner<L, T> where L: LinearStorageLen {}

#[cfg(kani)]
mod kani {
  use crate::collections::Vector;

  #[kani::proof]
  fn extend_from_iter() {
    let mut from = Vector::from_vec(kani::vec::any_vec::<u8, 128>());
    let to = kani::vec::any_vec::<u8, 128>();
    from.extend_from_iter(to.into_iter()).unwrap();
  }

  #[kani::proof]
  fn insert() {
    let elem = kani::any();
    let idx = kani::any();
    let mut vec = kani::vec::any_vec::<u8, 128>();
    let mut vector = Vector::from_vec(vec.clone());
    if idx > vec.len() {
      return;
    }
    vec.insert(idx, elem);
    vector.insert(idx, elem).unwrap();
    assert_eq!(vec.as_slice(), vector.as_slice());
  }

  #[kani::proof]
  fn push() {
    let elem = kani::any();
    let mut vec = kani::vec::any_vec::<u8, 128>();
    let mut vector = Vector::from_vec(vec.clone());
    vec.push(elem);
    vector.push(elem).unwrap();
    assert_eq!(vec.as_slice(), vector.as_slice());
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::collections::{LinearStorageLen, Vector};
  use core::{fmt::Formatter, marker::PhantomData};
  use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, SeqAccess, Visitor},
  };

  impl<'de, L, T> Deserialize<'de> for Vector<L, T>
  where
    L: LinearStorageLen,
    T: Deserialize<'de>,
  {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      struct LocalVisitor<L, T>(PhantomData<(L, T)>);

      impl<'de, L, T> Visitor<'de> for LocalVisitor<L, T>
      where
        L: LinearStorageLen,
        T: Deserialize<'de>,
      {
        type Value = Vector<L, T>;

        #[inline]
        fn expecting(&self, formatter: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
          formatter.write_fmt(format_args!("a vector of variable length"))
        }

        #[inline]
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
          A: SeqAccess<'de>,
        {
          let mut this = Vector::<L, T>::new();
          if let Some(hint) = seq.size_hint() {
            let _rslt = this.reserve(L::from_usize(hint).unwrap_or_default());
          }
          while let Some(elem) = seq.next_element()? {
            this.push(elem).map_err(|_err| {
              de::Error::invalid_length(
                this.len().usize(),
                &"vector need more data to be constructed",
              )
            })?;
          }
          Ok(this)
        }
      }

      deserializer.deserialize_seq(LocalVisitor::<L, T>(PhantomData))
    }
  }

  impl<L, T> Serialize for Vector<L, T>
  where
    usize: LinearStorageLen,
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
