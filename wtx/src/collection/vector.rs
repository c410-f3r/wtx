use crate::{
  collection::{
    ExpansionTy,
    linear_storage::{
      LinearStorage, linear_storage_mut::LinearStorageMut, linear_storage_slice::LinearStorageSlice,
    },
  },
  misc::{_unlikely_unreachable, Lease, LeaseMut, Wrapper},
};
use alloc::vec::{IntoIter, Vec};
use core::{
  borrow::{Borrow, BorrowMut},
  cmp::Ordering,
  fmt::{Debug, Display, Formatter},
  mem::ManuallyDrop,
  ops::{Deref, DerefMut},
  slice::{Iter, IterMut},
};

/// Errors of [Vector].
#[derive(Clone, Copy, Debug)]
pub enum VectorError {
  #[doc = doc_reserve_overflow!()]
  CapacityOverflow,
  #[doc = doc_many_elems_cap_overflow!()]
  ExtendFromSliceOverflow,
  #[doc = doc_many_elems_cap_overflow!()]
  ExtendFromSlicesOverflow,
  /// The index provided in the `insert` method is out of bounds.
  OutOfBoundsInsertIdx,
  #[doc = doc_single_elem_cap_overflow!()]
  PushOverflow,
  #[doc = doc_reserve_overflow!()]
  ReserveOverflow,
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
      VectorError::OutOfBoundsInsertIdx => 3,
      VectorError::PushOverflow => 4,
      VectorError::ReserveOverflow => 5,
      VectorError::VecOverflow => 6,
    }
  }
}

impl core::error::Error for VectorError {}

/// A wrapper around the std's vector.
pub struct Vector<T>(Inner<T>);

impl<T> Vector<T> {
  /// Constructs a new instance based on an arbitrary [Vec].
  ///
  /// ```rust
  /// let mut vec = wtx::collection::Vector::<u8>::from_vec(Vec::new());
  /// assert_eq!(vec.len(), 0);
  /// ```
  #[inline]
  pub const fn from_vec(vec: Vec<T>) -> Self {
    Self(Inner(vec))
  }

  /// Constructs a new, empty instance.
  ///
  /// ```rust
  /// let mut vec = wtx::collection::Vector::<u8>::new();
  /// assert_eq!(vec.len(), 0);
  /// ```
  #[inline]
  pub const fn new() -> Self {
    Self(Inner(Vec::new()))
  }

  /// Constructs a new, empty instance with at least the specified capacity.
  /// Constructs a new instance based on an arbitrary [Vec].
  ///
  /// ```rust
  /// let mut vec = wtx::collection::Vector::<u8>::with_capacity(2).unwrap();
  /// assert!(vec.capacity() >= 2);
  /// ```
  #[inline(always)]
  pub fn with_capacity(capacity: usize) -> crate::Result<Self> {
    Ok(Self(Inner(Vec::with_capacity(capacity))))
  }

  /// Constructs a new, empty instance with the exact specified capacity.
  ///
  /// ```rust
  /// let mut vec = wtx::collection::Vector::<u8>::with_exact_capacity(2).unwrap();
  /// assert_eq!(vec.capacity(), 2);
  /// ```
  #[inline(always)]
  pub fn with_exact_capacity(capacity: usize) -> crate::Result<Self> {
    let mut this = Self::new();
    this.reserve_exact(capacity)?;
    Ok(this)
  }

  /// Transfers memory ownership to the vector of the standard library.
  ///
  /// ```rust
  /// let vec = wtx::collection::Vector::<u8>::new();
  /// assert_eq!(vec.into_vec(), Vec::<u8>::new());
  /// ```
  #[inline]
  pub fn into_vec(self) -> Vec<T> {
    let mut wrapper = ManuallyDrop::new(self);
    let capacity = wrapper.capacity();
    let len = wrapper.len();
    // SAFETY: `self` has valid parameters that point to valid memory
    unsafe { Vec::from_raw_parts(wrapper.as_mut_ptr(), len, capacity) }
  }

  /// Vector of the standard library.
  #[inline]
  pub const fn vec_mut(&mut self) -> &mut Vec<T> {
    &mut self.0.0
  }
}

impl<T> Vector<T> {
  #[doc = from_cloneable_elem_doc!("Vector")]
  #[inline]
  pub fn from_cloneable_elem(len: usize, value: T) -> crate::Result<Self>
  where
    T: Clone,
  {
    Ok(Self(Inner::from_cloneable_elem(len, value)?))
  }

  #[doc = from_cloneable_slice_doc!("Vector")]
  #[inline]
  pub fn from_cloneable_slice(slice: &[T]) -> crate::Result<Self>
  where
    T: Clone,
  {
    Ok(Self(Inner::from_cloneable_slice(slice)?))
  }

  #[doc = from_copyable_slice_doc!("Vector")]
  #[inline]
  pub fn from_copyable_slice(slice: &[T]) -> crate::Result<Self>
  where
    T: Copy,
  {
    Ok(Self(Inner::from_copyable_slice(slice)?))
  }

  #[doc = from_iter_doc!("Vector", "[1, 2, 3]", "&[1, 2, 3]")]
  #[inline]
  pub fn from_iterator(iter: impl IntoIterator<Item = T>) -> crate::Result<Self> {
    Ok(Self(Inner::from_iterator(iter)?))
  }

  #[doc = as_ptr_doc!("Vector", "[1, 2, 3]")]
  #[inline]
  pub fn as_ptr(&self) -> *const T {
    self.0.as_ptr()
  }

  #[doc = as_ptr_mut_doc!()]
  #[inline]
  pub fn as_ptr_mut(&mut self) -> *mut T {
    self.0.as_ptr_mut()
  }

  #[doc = as_slice_doc!("Vector", "[1, 2, 3]", "[1, 2, 3]")]
  #[inline]
  pub fn as_slice(&self) -> &[T] {
    self.0.as_slice()
  }

  #[doc = as_slice_mut_doc!()]
  #[inline]
  pub fn as_slice_mut(&mut self) -> &mut [T] {
    self.0.as_slice_mut()
  }

  #[doc = capacity_doc!("Vector", "[1, 2, 3]")]
  #[inline]
  pub fn capacity(&self) -> usize {
    self.0.capacity()
  }

  #[doc = clear_doc!("Vector", "[1, 2, 3]")]
  #[inline]
  pub fn clear(&mut self) {
    self.0.clear();
  }

  #[doc = expand_doc!("Vector")]
  #[inline]
  pub fn expand(&mut self, et: ExpansionTy, value: T) -> crate::Result<()>
  where
    T: Clone,
  {
    self.0.expand(et, value)
  }

  #[doc = extend_from_cloneable_slice_doc!("Vector")]
  #[inline]
  pub fn extend_from_cloneable_slice(&mut self, other: &[T]) -> crate::Result<()>
  where
    T: Clone,
  {
    self.0.extend_from_cloneable_slice(other)
  }

  #[doc = extend_from_copyable_slice_doc!("Vector")]
  #[inline]
  pub fn extend_from_copyable_slice(&mut self, other: &[T]) -> crate::Result<()>
  where
    T: Copy,
  {
    self.0.extend_from_copyable_slice(other)
  }

  #[doc = extend_from_copyable_slice_doc!("Vector")]
  #[inline]
  pub fn extend_from_copyable_slices<E, I>(&mut self, others: I) -> crate::Result<usize>
  where
    E: Lease<[T]>,
    I: IntoIterator<Item = E>,
    I::IntoIter: Clone,
    T: Copy,
  {
    self.0.extend_from_copyable_slices(others)
  }

  #[doc = extend_from_iter_doc!("Vector", "[1, 2, 3]", "&[1, 2, 3]")]
  #[inline]
  pub fn extend_from_iter(&mut self, iter: impl IntoIterator<Item = T>) -> crate::Result<()> {
    self.0.extend_from_iter(iter)
  }

  #[doc = len_doc!()]
  #[inline]
  pub fn len(&self) -> usize {
    self.0.len()
  }

  #[doc = pop_doc!("Vector", "[1, 2, 3]", "[1, 2]")]
  #[inline]
  pub fn pop(&mut self) -> Option<T> {
    <[T] as LinearStorageSlice>::pop(&mut self.0)
  }

  #[doc = push_doc!("Vector", "1", "&[1]")]
  #[inline]
  pub fn push(&mut self, elem: T) -> crate::Result<()> {
    self.0.push(elem)
  }

  #[doc = remaining_doc!("Vector", "1")]
  #[inline]
  pub fn remaining(&self) -> usize {
    self.0.remaining()
  }

  #[doc = remove_doc!("Vector", "[1, 2, 3]", "[1, 3]")]
  #[inline]
  pub fn remove(&mut self, index: usize) -> Option<T> {
    <[T] as LinearStorageSlice>::remove(&mut self.0, index)
  }

  #[doc = reserve_doc!("Vector::<u8>")]
  #[inline]
  pub fn reserve(&mut self, additional: usize) -> crate::Result<()> {
    self.0.reserve(additional)
  }

  #[doc = reserve_exact_doc!("Vector::<u8>")]
  #[inline]
  pub fn reserve_exact(&mut self, additional: usize) -> crate::Result<()> {
    self.0.reserve_exact(additional)
  }

  #[doc = set_len_doc!()]
  #[inline]
  pub unsafe fn set_len(&mut self, new_len: usize) {
    // SAFETY: Up to the caller
    unsafe {
      self.0.set_len(new_len);
    }
  }

  #[doc = truncate_doc!("Vector", "[1, 2, 3]", "[1]")]
  #[inline]
  pub fn truncate(&mut self, new_len: usize) {
    let _rslt = <[T] as LinearStorageSlice>::truncate(&mut self.0, new_len);
  }
}

impl<T> Lease<[T]> for Vector<T> {
  #[inline]
  fn lease(&self) -> &[T] {
    self
  }
}

impl<T> Lease<Vector<T>> for Vector<T> {
  #[inline]
  fn lease(&self) -> &Vector<T> {
    self
  }
}

impl<T> LeaseMut<[T]> for Vector<T> {
  #[inline]
  fn lease_mut(&mut self) -> &mut [T] {
    self
  }
}

impl<T> LeaseMut<Vector<T>> for Vector<T> {
  #[inline]
  fn lease_mut(&mut self) -> &mut Vector<T> {
    self
  }
}

impl<T> AsMut<[T]> for Vector<T> {
  #[inline]
  fn as_mut(&mut self) -> &mut [T] {
    self
  }
}

impl<T> AsRef<[T]> for Vector<T> {
  #[inline]
  fn as_ref(&self) -> &[T] {
    self
  }
}

impl<T> Borrow<[T]> for Vector<T> {
  #[inline]
  fn borrow(&self) -> &[T] {
    self
  }
}

impl<T> BorrowMut<[T]> for Vector<T> {
  #[inline]
  fn borrow_mut(&mut self) -> &mut [T] {
    self
  }
}

impl<T> Clone for Vector<T>
where
  T: Clone,
{
  #[inline]
  #[track_caller]
  fn clone(&self) -> Self {
    let Ok(mut vector) = Self::with_capacity(self.len()) else {
      _unlikely_unreachable();
    };
    let _rslt = vector.extend_from_cloneable_slice(self);
    vector
  }

  #[inline]
  fn clone_from(&mut self, source: &Self) {
    self.truncate(source.len());
    let (init, tail) = source.split_at(self.len());
    self.clone_from_slice(init);
    if self.extend_from_cloneable_slice(tail).is_err() {
      _unlikely_unreachable();
    }
  }
}

impl<T> Debug for Vector<T>
where
  T: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    self.0.as_slice().fmt(f)
  }
}

impl<T> Default for Vector<T> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<T> Deref for Vector<T> {
  type Target = [T];

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.0.as_slice()
  }
}

impl<T> DerefMut for Vector<T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.0.as_slice_mut()
  }
}

impl<T> From<Vec<T>> for Vector<T> {
  #[inline]
  fn from(from: Vec<T>) -> Self {
    Vector::from_vec(from)
  }
}

impl<T> From<Vector<T>> for Vec<T> {
  #[inline]
  fn from(from: Vector<T>) -> Self {
    from.into_vec()
  }
}

impl<T> FromIterator<T> for Wrapper<crate::Result<Vector<T>>> {
  #[inline]
  fn from_iter<I>(iter: I) -> Self
  where
    I: IntoIterator<Item = T>,
  {
    Wrapper(Vector::from_iterator(iter))
  }
}

impl<T> Eq for Vector<T> where T: Eq {}

impl<T> IntoIterator for Vector<T> {
  type Item = T;
  type IntoIter = IntoIter<T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.into_vec().into_iter()
  }
}

impl<'any, T> IntoIterator for &'any Vector<T> {
  type Item = &'any T;
  type IntoIter = Iter<'any, T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

impl<'any, T> IntoIterator for &'any mut Vector<T> {
  type Item = &'any mut T;
  type IntoIter = IterMut<'any, T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.iter_mut()
  }
}

impl<T> Ord for Vector<T>
where
  T: Ord,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    (**self).cmp(other)
  }
}

impl<T> PartialEq for Vector<T>
where
  T: PartialEq,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

impl<T> PartialEq<[T]> for Vector<T>
where
  T: PartialEq,
{
  #[inline]
  fn eq(&self, other: &[T]) -> bool {
    **self == *other
  }
}

impl<T> PartialOrd for Vector<T>
where
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

impl core::fmt::Write for Vector<u8> {
  #[inline]
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    self.extend_from_copyable_slice(s.as_bytes()).map_err(|_err| core::fmt::Error)
  }
}

#[cfg(feature = "std")]
impl std::io::Write for Vector<u8> {
  #[inline]
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    self
      .extend_from_copyable_slice(buf)
      .map_err(|err| std::io::Error::new(std::io::ErrorKind::StorageFull, err))?;
    Ok(buf.len())
  }

  #[inline]
  fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
    let mut fun = || {
      let len: usize = bufs.iter().map(|b| b.len()).sum();
      self.reserve(len)?;
      self.extend_from_copyable_slices(bufs)
    };
    fun().map_err(|err| std::io::Error::new(std::io::ErrorKind::StorageFull, err))
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

struct Inner<T>(Vec<T>);

impl<T> LinearStorage<T> for Inner<T> {
  type Len = usize;
  type Slice = [T];

  #[inline]
  fn as_ptr(&self) -> *const T {
    self.0.as_ptr()
  }

  #[inline]
  fn capacity(&self) -> Self::Len {
    self.0.capacity()
  }

  #[inline]
  fn len(&self) -> Self::Len {
    self.0.len()
  }
}

impl<T> LinearStorageMut<T> for Inner<T> {
  #[inline]
  fn as_ptr_mut(&mut self) -> *mut T {
    self.0.as_mut_ptr()
  }

  #[inline]
  fn reserve(&mut self, additional: Self::Len) -> crate::Result<()> {
    self.0.try_reserve(additional).map_err(|_err| VectorError::ReserveOverflow)?;
    Ok(())
  }

  #[inline]
  fn reserve_exact(&mut self, additional: Self::Len) -> crate::Result<()> {
    self.0.try_reserve_exact(additional).map_err(|_err| VectorError::ReserveOverflow)?;
    Ok(())
  }

  #[inline]
  unsafe fn set_len(&mut self, new_len: Self::Len) {
    // SAFETY: Up to the caller
    unsafe { self.0.set_len(new_len) }
  }
}

impl<T> Default for Inner<T> {
  fn default() -> Self {
    Self(Vec::new())
  }
}

#[cfg(kani)]
mod kani {
  use crate::collection::Vector;

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
  use crate::collection::{LinearStorageLen, Vector};
  use core::{fmt::Formatter, marker::PhantomData};
  use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, SeqAccess, Visitor},
  };

  impl<'de, T> Deserialize<'de> for Vector<T>
  where
    T: Deserialize<'de>,
  {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      struct LocalVisitor<T>(PhantomData<T>);

      impl<'de, T> Visitor<'de> for LocalVisitor<T>
      where
        T: Deserialize<'de>,
      {
        type Value = Vector<T>;

        #[inline]
        fn expecting(&self, formatter: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
          formatter.write_fmt(format_args!("a vector of variable length"))
        }

        #[inline]
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
          A: SeqAccess<'de>,
        {
          let mut this = Vector::<T>::new();
          while let Some(elem) = seq.next_element()? {
            this.push(elem).map_err(|_err| {
              de::Error::invalid_length(this.len(), &"vector need more data to be constructed")
            })?;
          }
          Ok(this)
        }
      }

      deserializer.deserialize_seq(LocalVisitor::<T>(PhantomData))
    }
  }

  impl<T> Serialize for Vector<T>
  where
    usize: LinearStorageLen,
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
