use crate::misc::{BufferParam, Lease, LeaseMut, _unlikely_elem};
use alloc::vec::{Drain, IntoIter, Vec};
use core::{
  borrow::{Borrow, BorrowMut},
  fmt::{Debug, Display, Formatter},
  hint::assert_unchecked,
  ops::{Deref, DerefMut, RangeBounds},
  ptr, slice,
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
#[cfg_attr(feature = "test-strategy", derive(test_strategy::Arbitrary))]
#[cfg_attr(feature = "test-strategy", arbitrary(bound(T: proptest::arbitrary::Arbitrary + 'static)))]
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Vector<T> {
  data: Vec<T>,
}

impl<T> Vector<T> {
  /// Constructs a new instance with elements provided by `iter`.
  ///
  /// ```rust
  /// let mut vec = wtx::misc::Vector::from_iter(0u8..2).unwrap();
  /// assert_eq!(vec.as_slice(), &[0, 1]);
  /// ```
  #[expect(clippy::should_implement_trait, reason = "Std trait is infallible")]
  #[inline]
  pub fn from_iter(iter: impl IntoIterator<Item = T>) -> Result<Self, VectorError> {
    let mut this = Self::new();
    this.extend_from_iter(iter)?;
    Ok(this)
  }

  /// Constructs a new instance based on an arbitrary [Vec].
  ///
  /// ```rust
  /// let mut vec = wtx::misc::Vector::<u8>::from_vec(Vec::new());
  /// assert_eq!(vec.len(), 0);
  /// ```
  #[inline]
  pub const fn from_vec(data: Vec<T>) -> Self {
    Self { data }
  }

  /// Constructs a new, empty instance.
  ///
  /// ```rust
  /// let mut vec = wtx::misc::Vector::<u8>::new();
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
  /// let mut vec = wtx::misc::Vector::<u8>::with_capacity(2).unwrap();
  /// assert!(vec.capacity() >= 2);
  /// ```
  #[inline(always)]
  pub fn with_capacity(cap: usize) -> Result<Self, VectorError> {
    let this = Self { data: Vec::with_capacity(cap) };
    // SAFETY: `len` will never be greater than the current capacity
    unsafe {
      assert_unchecked(this.data.capacity() >= this.data.len());
    }
    Ok(this)
  }

  /// Returns an unsafe mutable pointer to the vector's buffer, or a dangling
  /// raw pointer valid for zero sized reads if the vector didn't allocate.
  #[inline]
  pub fn as_mut_ptr(&mut self) -> *mut T {
    self.data.as_mut_ptr()
  }

  /// Extracts a slice containing the entire mutable vector.
  #[inline]
  pub fn as_mut_slice(&mut self) -> &mut [T] {
    self.data.as_mut_slice()
  }

  /// Returns a raw pointer to the vector's buffer, or a dangling raw pointer
  /// valid for zero sized reads if the vector didn't allocate.
  #[inline]
  pub fn as_ptr(&self) -> *const T {
    self.data.as_ptr()
  }

  /// Extracts a slice containing the entire vector.
  #[inline]
  pub fn as_slice(&self) -> &[T] {
    self.data.as_slice()
  }

  /// Returns the total number of elements the vector can hold without reallocating.
  ///
  /// ```rust
  /// let mut vec = wtx::misc::Vector::new();
  /// assert_eq!(vec.capacity(), 0);
  /// vec.push(1u8);
  /// assert!(vec.capacity() >= 1);
  /// ```
  #[inline]
  pub fn capacity(&self) -> usize {
    self.data.capacity()
  }

  /// Clears the vector, removing all values.
  ///
  /// ```rust
  /// let mut vec = wtx::misc::Vector::new();
  /// vec.push(1u8);
  /// assert_eq!(vec.len(), 1);
  /// vec.clear();
  /// assert_eq!(vec.len(), 0);
  /// ```
  #[inline]
  pub fn clear(&mut self) {
    self.data.clear();
  }

  /// Clears the vector, removing all values.
  #[inline]
  pub fn drain<R>(&mut self, range: R) -> Drain<'_, T>
  where
    R: RangeBounds<usize>,
  {
    self.data.drain(range)
  }

  /// Clones and appends all elements in the iterator.
  #[inline]
  pub fn extend_from_iter(&mut self, ii: impl IntoIterator<Item = T>) -> Result<(), VectorError> {
    let iter = ii.into_iter();
    self.data.reserve(iter.size_hint().0);
    for elem in iter {
      self.push(elem)?;
    }
    Ok(())
  }

  /// Constructs a new instance with elements provided by `iter`.
  #[inline]
  pub fn insert(&mut self, idx: usize, elem: T) -> Result<(), VectorError> {
    let len = self.len();
    if idx > len {
      return _unlikely_elem(Err(VectorError::OutOfBoundsInsertIdx));
    }
    self.reserve(1)?;
    // SAFETY: Top-level check ensures bounds
    let ptr = unsafe { self.as_mut_ptr().add(idx) };
    if idx < len {
      // SAFETY: Top-level check ensures bounds
      let diff = unsafe { len.unchecked_sub(idx) };
      // SAFETY: `reserve` allocated one more element
      let dst = unsafe { ptr.add(1) };
      // SAFETY: Up to the other elements
      unsafe {
        ptr::copy(ptr, dst, diff);
      }
    }
    // SAFETY: Write it in, overwriting the first copy of the `index`th element
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

  /// Removes the last element from a vector and returns it, or [None] if it is empty.
  #[inline]
  pub fn pop(&mut self) -> Option<T> {
    self.data.pop()
  }

  /// Appends an element to the back of the collection.
  #[inline]
  pub fn push(&mut self, value: T) -> Result<(), VectorError> {
    self.reserve(1).map_err(|_err| VectorError::PushOverflow)?;
    let len = self.data.len();
    // SAFETY: `len` points to valid memory
    let dst = unsafe { self.data.as_mut_ptr().add(len) };
    // SAFETY: `dst` points to valid memory
    unsafe {
      ptr::write(dst, value);
    }
    // SAFETY: top-level check ensures capacity
    let new_len = unsafe { len.unchecked_add(1) };
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(new_len);
    }
    Ok(())
  }

  /// Shortens the vector, keeping the first len elements and dropping the rest.
  #[inline]
  pub fn remove(&mut self, idx: usize) -> Option<T> {
    if idx >= self.data.len() {
      return None;
    }
    Some(self.data.remove(idx))
  }

  /// Reserves capacity for at least `additional` more elements to be inserted
  /// in the given instance. The collection may reserve more space to
  /// speculatively avoid frequent reallocations. After calling `reserve`,
  /// capacity will be greater than or equal to `self.len() + additional`.
  /// Does nothing if capacity is already sufficient.
  #[inline(always)]
  pub fn reserve(&mut self, additional: usize) -> Result<(), VectorError> {
    self.data.try_reserve(additional).map_err(|_err| VectorError::ReserveOverflow)?;
    // SAFETY: `len` will never be greater than the current capacity
    unsafe {
      assert_unchecked(self.data.capacity() >= self.data.len());
    }
    Ok(())
  }

  /// Shortens the vector, keeping the first len elements and dropping the rest.
  #[inline]
  pub fn truncate(&mut self, len: usize) {
    self.data.truncate(len);
  }

  /// Forces the length of the vector to `new_len`.
  ///
  /// # Safety
  ///
  /// - `new_len` must be less than or equal to the capacity.
  /// - The elements at `prev_len..new_len` must be initialized.
  #[inline]
  pub(crate) unsafe fn set_len(&mut self, new_len: usize) {
    // Safety: up to the caller
    unsafe {
      self.data.set_len(new_len);
    }
  }
}

impl<T> Vector<T>
where
  T: Clone,
{
  /// Constructs a new instance with elements provided by `iter`.
  #[inline]
  pub fn from_cloneable_elem(len: usize, value: T) -> Result<Self, VectorError> {
    let mut this = Self::with_capacity(len)?;
    this.expand(BufferParam::Len(len), value)?;
    Ok(this)
  }

  /// Resizes the instance in-place so that the current length is equal to `bp`.
  ///
  /// Does nothing if the calculated length is equal or less than the current length.
  #[inline(always)]
  pub fn expand(&mut self, bp: BufferParam, value: T) -> Result<(), VectorError> {
    let len = self.data.len();
    let Some((additional, new_len)) = bp.params(len) else {
      return Ok(());
    };
    self.reserve(additional)?;
    // SAFETY: there are initialized elements until `len`
    let ptr = unsafe { self.data.as_mut_ptr().add(len) };
    // SAFETY: memory has been allocated
    unsafe {
      slice::from_raw_parts_mut(ptr, additional).fill(value);
    }
    // SAFETY: elements have been initialized
    unsafe {
      self.data.set_len(new_len);
    }
    Ok(())
  }
}

impl<T> Vector<T>
where
  T: Copy,
{
  /// Constructs a new instance with elements provided by `slice`.
  #[inline]
  pub fn from_slice(slice: &[T]) -> Result<Self, VectorError> {
    let mut this = Self::new();
    this.extend_from_slice(slice)?;
    Ok(this)
  }

  /// Iterates over the slice `other`, copies each element, and then appends
  /// it to this vector. The `other` slice is traversed in-order.
  #[inline]
  pub fn extend_from_slice(&mut self, other: &[T]) -> Result<(), VectorError> {
    self.reserve(other.len()).map_err(|_err| VectorError::ExtendFromSliceOverflow)?;
    // SAFETY: memory has been allocated
    unsafe {
      self.do_extend_from_slice(other);
    }
    Ok(())
  }

  /// Generalization of [`Self::extend_from_slice`].
  ///
  /// Returns the sum of the lengths of all slices.
  #[inline(always)]
  pub fn extend_from_slices<'iter, I>(&mut self, others: I) -> Result<usize, VectorError>
  where
    I: IntoIterator<Item = &'iter [T]>,
    I::IntoIter: Clone,
    T: 'iter,
  {
    let mut len: usize = 0;
    let iter = others.into_iter();
    for other in iter.clone() {
      let Some(curr_len) = len.checked_add(other.len()) else {
        return Err(VectorError::ExtendFromSlicesOverflow);
      };
      len = curr_len;
    }
    self.reserve(len).map_err(|_err| VectorError::ExtendFromSlicesOverflow)?;
    for other in iter {
      // SAFETY: memory has been allocated
      unsafe {
        self.do_extend_from_slice(other);
      }
    }
    Ok(len)
  }

  #[inline]
  unsafe fn do_extend_from_slice(&mut self, other: &[T]) {
    let len = self.data.len();
    let other_len = other.len();
    let new_len = len.wrapping_add(other_len);
    // SAFETY: there are initialized elements until `len`
    let dst = unsafe { self.data.as_mut_ptr().add(len) };
    // SAFETY: caller must ensure allocated space
    unsafe {
      ptr::copy_nonoverlapping(other.as_ptr(), dst, other_len);
    }
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(new_len);
    }
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

#[cfg(feature = "serde")]
mod serde {
  use crate::misc::Vector;
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

#[cfg(feature = "quick-protobuf")]
impl quick_protobuf::writer::WriterBackend for &mut Vector<u8> {
  #[inline]
  fn pb_write_u8(&mut self, x: u8) -> quick_protobuf::Result<()> {
    self.push(x).map_err(|_err| quick_protobuf::Error::UnexpectedEndOfBuffer)
  }

  #[inline]
  fn pb_write_u32(&mut self, x: u32) -> quick_protobuf::Result<()> {
    self
      .extend_from_slice(&x.to_be_bytes())
      .map_err(|_err| quick_protobuf::Error::UnexpectedEndOfBuffer)
  }

  #[inline]
  fn pb_write_i32(&mut self, x: i32) -> quick_protobuf::Result<()> {
    self
      .extend_from_slice(&x.to_be_bytes())
      .map_err(|_err| quick_protobuf::Error::UnexpectedEndOfBuffer)
  }

  #[inline]
  fn pb_write_f32(&mut self, x: f32) -> quick_protobuf::Result<()> {
    self
      .extend_from_slice(&x.to_be_bytes())
      .map_err(|_err| quick_protobuf::Error::UnexpectedEndOfBuffer)
  }

  #[inline]
  fn pb_write_u64(&mut self, x: u64) -> quick_protobuf::Result<()> {
    self
      .extend_from_slice(&x.to_be_bytes())
      .map_err(|_err| quick_protobuf::Error::UnexpectedEndOfBuffer)
  }

  #[inline]
  fn pb_write_i64(&mut self, x: i64) -> quick_protobuf::Result<()> {
    self
      .extend_from_slice(&x.to_be_bytes())
      .map_err(|_err| quick_protobuf::Error::UnexpectedEndOfBuffer)
  }

  #[inline]
  fn pb_write_f64(&mut self, x: f64) -> quick_protobuf::Result<()> {
    self
      .extend_from_slice(&x.to_be_bytes())
      .map_err(|_err| quick_protobuf::Error::UnexpectedEndOfBuffer)
  }

  #[inline]
  fn pb_write_all(&mut self, buf: &[u8]) -> quick_protobuf::Result<()> {
    self.extend_from_slice(buf).map_err(|_err| quick_protobuf::Error::UnexpectedEndOfBuffer)
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

impl<T> IntoIterator for Vector<T> {
  type Item = T;
  type IntoIter = IntoIter<T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.data.into_iter()
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

impl core::fmt::Write for Vector<u8> {
  #[inline]
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    self.extend_from_slice(s.as_bytes()).map_err(|_err| core::fmt::Error)
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

#[cfg(feature = "cl-aux")]
mod cl_aux {
  use crate::misc::Vector;
  use cl_aux::{Capacity, Clear, Extend, Push, SingleTypeStorage, Truncate, WithCapacity};

  impl<T> Capacity for Vector<T> {
    #[inline]
    fn capacity(&self) -> usize {
      self.capacity()
    }
  }

  impl<T> Clear for Vector<T> {
    #[inline]
    fn clear(&mut self) {
      self.clear();
    }
  }

  impl<T> Extend<T> for Vector<T> {
    type Error = crate::Error;

    #[inline]
    fn extend(&mut self, into_iter: impl IntoIterator<Item = T>) -> Result<(), Self::Error> {
      self.extend_from_iter(into_iter)?;
      Ok(())
    }
  }

  impl<T> Push<T> for Vector<T> {
    type Error = crate::Error;

    #[inline]
    fn push(&mut self, input: T) -> Result<(), Self::Error> {
      self.push(input)?;
      Ok(())
    }
  }

  impl<T> SingleTypeStorage for Vector<T> {
    type Item = T;
  }

  impl<T> Truncate for Vector<T> {
    type Input = usize;

    #[inline]
    fn truncate(&mut self, input: Self::Input) {
      (*self).truncate(input);
    }
  }

  impl<T> WithCapacity for Vector<T> {
    type Error = crate::Error;
    type Input = usize;

    #[inline]
    fn with_capacity(input: Self::Input) -> Self {
      Vector::with_capacity(input).unwrap()
    }
  }
}

#[cfg(feature = "_proptest")]
#[cfg(test)]
mod _proptest {
  use crate::misc::Vector;
  use alloc::vec::Vec;

  #[test_strategy::proptest]
  fn insert(elem: u8, idx: usize, mut vec: Vec<u8>) {
    let mut vector = Vector::from_vec(vec.clone());
    if idx > vec.len() {
      return Ok(());
    }
    vec.insert(idx, elem);
    vector.insert(idx, elem).unwrap();
    assert_eq!(vec.as_slice(), vector.as_slice());
  }

  #[test_strategy::proptest]
  fn push(elem: u8, mut vec: Vec<u8>) {
    let mut vector = Vector::from_vec(vec.clone());
    vec.push(elem);
    vector.push(elem).unwrap();
    assert_eq!(vec.as_slice(), vector.as_slice());
  }
}
