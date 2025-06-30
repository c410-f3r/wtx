use crate::{
  collection::{IndexedStorage, IndexedStorageMut},
  misc::{Lease, LeaseMut, Wrapper, hints::unlikely_elem},
};
use alloc::vec::{Drain, IntoIter, Vec};
use core::{
  borrow::{Borrow, BorrowMut},
  cmp::Ordering,
  fmt::{Debug, Display, Formatter},
  hint::assert_unchecked,
  ops::{Deref, DerefMut, RangeBounds},
  ptr,
  slice::{Iter, IterMut},
};

/// Errors of [Vector].
#[derive(Clone, Copy, Debug)]
pub enum VectorError {
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
      VectorError::ExtendFromSliceOverflow => 0,
      VectorError::ExtendFromSlicesOverflow => 1,
      VectorError::OutOfBoundsInsertIdx => 2,
      VectorError::PushOverflow => 3,
      VectorError::ReserveOverflow => 4,
    }
  }
}

impl core::error::Error for VectorError {}

/// A wrapper around the std's vector.
//#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone)]
#[repr(transparent)]
pub struct Vector<T> {
  data: Vec<T>,
}

impl<T> Vector<T> {
  /// Constructs a new instance based on an arbitrary [Vec].
  ///
  /// ```rust
  /// let mut vec = wtx::collection::Vector::<u8>::from_vec(Vec::new());
  /// assert_eq!(vec.len(), 0);
  /// ```
  #[inline]
  pub const fn from_vec(data: Vec<T>) -> Self {
    Self { data }
  }

  /// Constructs a new, empty instance.
  ///
  /// ```rust
  /// let mut vec = wtx::collection::Vector::<u8>::new();
  /// assert_eq!(vec.len(), 0);
  /// ```
  #[inline]
  pub const fn new() -> Self {
    Self::from_vec(Vec::new())
  }

  /// Constructs a new, empty instance with at least the specified capacity.
  /// Constructs a new instance based on an arbitrary [Vec].
  ///
  /// ```rust
  /// use wtx::collection::{IndexedStorage, IndexedStorageMut};
  /// let mut vec = wtx::collection::Vector::<u8>::with_capacity(2).unwrap();
  /// assert!(vec.capacity() >= 2);
  /// ```
  #[inline(always)]
  pub fn with_capacity(cap: usize) -> crate::Result<Self> {
    let this = Self { data: Vec::with_capacity(cap) };
    // SAFETY: `len` will never be greater than the current capacity
    unsafe {
      assert_unchecked(this.data.capacity() >= this.data.len());
    }
    Ok(this)
  }

  /// Constructs a new, empty instance with the exact specified capacity.
  ///
  /// ```rust
  /// use wtx::collection::IndexedStorage;
  /// let mut vec = wtx::collection::Vector::<u8>::with_exact_capacity(2).unwrap();
  /// assert_eq!(vec.capacity(), 2);
  /// ```
  #[inline(always)]
  pub fn with_exact_capacity(cap: usize) -> crate::Result<Self> {
    let mut this = Self { data: Vec::new() };
    this.reserve_exact(cap)?;
    Ok(this)
  }

  /// Mutable reference of the underlying std vector.
  #[inline]
  pub fn as_vec_mut(&mut self) -> &mut Vec<T> {
    &mut self.data
  }

  /// Removes all but the first of consecutive elements in the vector satisfying a given equality
  /// relation.
  #[inline]
  pub fn dedup_by<F>(&mut self, same_bucket: F)
  where
    F: FnMut(&mut T, &mut T) -> bool,
  {
    self.data.dedup_by(same_bucket);
  }

  /// Clears the vector, removing all values.
  #[inline]
  pub fn drain<R>(&mut self, range: R) -> Drain<'_, T>
  where
    R: RangeBounds<usize>,
  {
    self.data.drain(range)
  }

  /// Inserts an element at position index within the instance, shifting all elements after it to
  /// the right.
  ///
  /// ```rust
  /// use wtx::collection::{IndexedStorage, IndexedStorageMut};
  /// let mut vec = wtx::collection::Vector::from_iter(1u8..4).unwrap();
  /// vec.insert(1, 4);
  /// assert_eq!(vec.as_slice(), [1, 4, 2, 3]);
  /// vec.insert(4, 5);
  /// assert_eq!(vec.as_slice(), [1, 4, 2, 3, 5]);
  /// ```
  #[inline]
  pub fn insert(&mut self, idx: usize, elem: T) -> crate::Result<()> {
    let len = self.len();
    if idx > len {
      return unlikely_elem(Err(VectorError::OutOfBoundsInsertIdx.into()));
    }
    self.reserve(1)?;
    // SAFETY: top-level check ensures bounds
    let ptr = unsafe { self.as_ptr_mut().add(idx) };
    if idx < len {
      // SAFETY: top-level check ensures bounds
      let diff = unsafe { len.unchecked_sub(idx) };
      // SAFETY: `reserve` allocated one more element
      let dst = unsafe { ptr.add(1) };
      // SAFETY: up to the other elements
      unsafe {
        ptr::copy(ptr, dst, diff);
      }
    }
    // SAFETY: write it in, overwriting the first copy of the `index`th element
    unsafe {
      ptr::write(ptr, elem);
    }
    // SAFETY: top-level check ensures bounds
    let new_len = unsafe { len.unchecked_add(1) };
    // SAFETY: `reserve` already handled memory capacity
    unsafe {
      self.set_len(new_len);
    }
    Ok(())
  }

  /// Shortens the vector, keeping the first len elements and dropping the rest.
  ///
  /// ```rust
  /// use wtx::collection::{IndexedStorage, IndexedStorageMut};
  /// let mut vec = wtx::collection::Vector::from_iter(1u8..4).unwrap();
  /// assert_eq!(vec.remove(1), Some(2));
  /// assert_eq!(vec.as_slice(), [1, 3]);
  /// ```
  #[inline]
  pub fn remove(&mut self, idx: usize) -> Option<T> {
    if idx >= self.data.len() {
      return None;
    }
    Some(self.data.remove(idx))
  }

  /// Tries to reserve the minimum capacity for at least `additional`
  /// elements to be inserted in the given instance. Unlike [`Self::reserve`],
  /// this will not deliberately over-allocate to speculatively avoid frequent
  /// allocations.
  ///
  /// ```rust
  /// use wtx::collection::{IndexedStorage, IndexedStorageMut};
  /// let mut vec = wtx::collection::Vector::<u8>::new();
  /// vec.reserve(10);
  /// assert!(vec.capacity() >= 10);
  /// ```
  #[inline(always)]
  pub fn reserve_exact(&mut self, additional: usize) -> crate::Result<()> {
    self.data.try_reserve_exact(additional).map_err(|_err| VectorError::ReserveOverflow)?;
    // SAFETY: `len` will never be greater than the current capacity
    unsafe {
      assert_unchecked(self.data.capacity() >= self.data.len());
    }
    Ok(())
  }

  /// Retains only the elements specified by the predicate.
  ///
  /// In other words, remove all elements `e` for which `f(&e)` returns `false`.
  /// This method operates in place, visiting each element exactly once in the
  /// original order, and preserves the order of the retained elements.
  ///
  /// # Examples
  ///
  /// ```
  /// use wtx::collection::{IndexedStorage, IndexedStorageMut};
  /// let mut vec = wtx::collection::Vector::from_iter(1u8..5).unwrap();
  /// vec.retain(|&x| x % 2 == 0);
  /// assert_eq!(vec.as_slice(), [2, 4]);
  /// ```
  #[inline(always)]
  pub fn retain(&mut self, f: impl FnMut(&T) -> bool) {
    self.data.retain(f);
  }
}

impl<T> IndexedStorage<T> for Vector<T> {
  type Len = usize;
  type Slice = [T];

  #[inline]
  fn as_ptr(&self) -> *const T {
    self.data.as_ptr().cast()
  }

  #[inline]
  fn capacity(&self) -> Self::Len {
    self.data.capacity()
  }

  #[inline]
  fn len(&self) -> Self::Len {
    self.data.len()
  }
}

impl<T> IndexedStorageMut<T> for Vector<T> {
  #[inline]
  fn as_ptr_mut(&mut self) -> *mut T {
    self.data.as_mut_ptr().cast()
  }

  #[inline]
  fn pop(&mut self) -> Option<T> {
    self.data.pop()
  }

  #[inline]
  fn reserve(&mut self, additional: Self::Len) -> crate::Result<()> {
    self.data.try_reserve(additional).map_err(|_err| VectorError::ReserveOverflow)?;
    Ok(())
  }

  #[inline]
  unsafe fn set_len(&mut self, new_len: Self::Len) {
    // SAFETY: delegated to `data`
    unsafe {
      self.data.set_len(new_len);
    }
  }

  #[inline]
  fn truncate(&mut self, new_len: Self::Len) {
    self.data.truncate(new_len);
  }
}

impl<T> Lease<[T]> for Vector<T> {
  #[inline]
  fn lease(&self) -> &[T] {
    self.data.as_slice()
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
    self.as_slice()
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

impl<T> Debug for Vector<T>
where
  T: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    self.data.fmt(f)
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
    self.data.as_slice()
  }
}

impl<T> DerefMut for Vector<T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.data.as_mut_slice()
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
    from.data
  }
}

impl<T> FromIterator<T> for Wrapper<crate::Result<Vector<T>>> {
  #[inline]
  fn from_iter<I>(iter: I) -> Self
  where
    I: IntoIterator<Item = T>,
  {
    Wrapper(Vector::from_iter(iter))
  }
}

impl<T> Eq for Vector<T> where T: Eq {}

impl<T> IntoIterator for Vector<T> {
  type Item = T;
  type IntoIter = IntoIter<T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.data.into_iter()
  }
}

impl<'any, T> IntoIterator for &'any Vector<T> {
  type Item = &'any T;
  type IntoIter = Iter<'any, T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.data.iter()
  }
}

impl<'any, T> IntoIterator for &'any mut Vector<T> {
  type Item = &'any mut T;
  type IntoIter = IterMut<'any, T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.data.iter_mut()
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

impl<T> Ord for Vector<T>
where
  T: Ord,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    (**self).cmp(other)
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
    self.data.write(buf)
  }

  #[inline]
  fn flush(&mut self) -> std::io::Result<()> {
    self.data.flush()
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
  use crate::collection::Vector;
  use alloc::vec::Vec;
  use serde::{Deserialize, Deserializer, Serialize, Serializer};

  impl<'de, T> Deserialize<'de> for Vector<T>
  where
    T: Deserialize<'de>,
  {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      Ok(Self::from_vec(Vec::deserialize(deserializer)?))
    }
  }

  impl<T> Serialize for Vector<T>
  where
    T: Serialize,
  {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      self.data.serialize(serializer)
    }
  }
}
