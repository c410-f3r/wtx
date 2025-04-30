#![expect(clippy::mem_forget, reason = "out-of-bounds elements are manually dropped")]

use crate::misc::{Lease, LeaseMut, Usize, Wrapper, char_slice};
use core::{
  cmp::Ordering,
  fmt::{self, Debug, Formatter},
  mem::{self, MaybeUninit, needs_drop},
  ops::{Deref, DerefMut},
  ptr, slice,
};

/// Errors of [`ArrayVector`].
#[derive(Debug)]
pub enum ArrayVectorError {
  #[doc = doc_many_elems_cap_overflow!()]
  ExtendFromSliceOverflow,
  /// Inner array is not totally full
  IntoInnerIncomplete,
  #[doc = doc_single_elem_cap_overflow!()]
  PushOverflow,
}

/// A wrapper around the std's vector with some additional methods to manipulate copyable data.
pub struct ArrayVector<T, const N: usize> {
  len: u32,
  data: [MaybeUninit<T>; N],
}

impl<T, const N: usize> ArrayVector<T, N> {
  const _INSTANCE_CHECK: () = {
    assert!(N <= Usize::from_u32(u32::MAX).into_usize());
  };
  const N_U32: u32 = {
    let [a, b, c, d, ..] = Usize::from_usize(N).into_u64().to_le_bytes();
    u32::from_le_bytes([a, b, c, d])
  };
  const NEEDS_DROP: bool = needs_drop::<T>();

  /// Constructs a new instance from a fully initialized array.
  #[inline]
  pub fn from_array(array: [T; N]) -> Self {
    let mut this = Self::new();
    this.len = Self::N_U32;
    // SAFETY: The inner `data` as well as the provided `array` have the same layout in different
    // memory regions
    unsafe {
      ptr::copy_nonoverlapping(array.as_ptr(), this.as_ptr_mut(), N);
    }
    mem::forget(array);
    this
  }

  /// Constructs a new instance from an iterator.
  ///
  /// The iterator is capped at `N`.
  #[expect(clippy::should_implement_trait, reason = "The std trait is infallible")]
  #[inline]
  pub fn from_iter(iter: impl IntoIterator<Item = T>) -> crate::Result<Self> {
    let mut this = Self::new();
    for elem in iter.into_iter().take(N) {
      let _rslt = this.push(elem);
    }
    Ok(this)
  }

  /// Constructs a new instance reusing `data` elements optionally delimited by `len`.
  ///
  /// The actual length will be the smallest value among `M`, `N` and `len`
  #[inline]
  pub fn from_parts<const M: usize>(mut data: [T; M], len: Option<u32>) -> Self {
    let mut actual_len = u32::try_from(M).unwrap_or(u32::MAX).min(Self::N_U32);
    if let Some(elem) = len {
      actual_len = actual_len.min(elem);
    }
    let actual_len_usize = Usize::from_u32(actual_len).into_usize();
    let mut this = Self::new();
    this.len = actual_len;
    // SAFETY: The inner `data` as well as the provided `data` have the same layout in different
    // memory regions
    unsafe {
      ptr::copy_nonoverlapping(data.as_ptr(), this.as_ptr_mut(), actual_len_usize);
    }
    if Self::NEEDS_DROP {
      let diff_opt = data.len().checked_sub(actual_len_usize);
      if let Some(diff @ 1..=usize::MAX) = diff_opt {
        // SAFETY: Indices are within bounds
        unsafe {
          drop_elements(diff, actual_len, data.as_mut_ptr());
        }
      }
    }
    mem::forget(data);
    this
  }

  /// Constructs a new empty instance.
  #[inline]
  pub const fn new() -> Self {
    Self { len: 0, data: [const { MaybeUninit::uninit() }; N] }
  }

  /// Extracts a slice containing the entire vector.
  #[inline]
  pub const fn as_slice(&self) -> &[T] {
    // SAFETY: `len` ensures initialized elements
    unsafe { slice::from_raw_parts(self.as_ptr(), Usize::from_u32(self.len).into_usize()) }
  }

  /// The number of elements that can be stored.
  #[inline]
  pub const fn capacity(&self) -> u32 {
    Self::N_U32
  }

  /// Clears the vector, removing all values.
  #[inline]
  pub fn clear(&mut self) {
    self.truncate(0);
  }

  /// Iterates over the slice `other`, copies each element, and then appends
  /// it to this vector. The `other` slice is traversed in-order.
  #[inline]
  pub fn extend_from_iter(&mut self, iter: impl IntoIterator<Item = T>) -> crate::Result<()> {
    for elem in iter {
      self.push(elem)?;
    }
    Ok(())
  }

  /// Return the inner fixed size array, if the capacity is full.
  #[inline]
  pub fn into_inner(self) -> crate::Result<[T; N]> {
    if Usize::from_u32(self.len).into_usize() >= N {
      // SAFETY: All elements are initialized
      Ok(unsafe { ptr::read(self.data.as_ptr().cast()) })
    } else {
      Err(ArrayVectorError::IntoInnerIncomplete.into())
    }
  }

  /// Returns the number of elements in the vector, also referred to as its ‘length’.
  #[inline]
  pub const fn len(&self) -> u32 {
    self.len
  }

  /// Shortens the vector, removing the last element.
  #[inline]
  pub const fn pop(&mut self) -> Option<T> {
    if let Some(new_len) = self.len.checked_sub(1) {
      self.len = new_len;
      // SAFETY: `new_len` is within bounds
      Some(unsafe { self.get_owned(new_len) })
    } else {
      None
    }
  }

  /// How many elements can be added to this collection.
  #[inline]
  pub const fn remaining(&self) -> u32 {
    self.capacity().wrapping_sub(self.len)
  }

  /// Appends an element to the back of the collection.
  #[inline]
  pub fn push(&mut self, value: T) -> crate::Result<()> {
    self.do_push(value).map_err(|_err| ArrayVectorError::PushOverflow)?;
    Ok(())
  }

  /// Shortens the vector, keeping the first `len` elements.
  #[inline]
  pub fn truncate(&mut self, new_len: u32) {
    let len = self.len;
    let Some(diff @ 1..=u32::MAX) = len.checked_sub(new_len) else {
      return;
    };
    self.len = new_len;
    if Self::NEEDS_DROP {
      // SAFETY: Indices are within bounds
      unsafe {
        drop_elements(*Usize::from(diff), new_len, self.as_ptr_mut());
      }
    }
  }

  #[inline]
  const fn as_ptr(&self) -> *const T {
    self.data.as_ptr().cast()
  }

  #[inline]
  const fn as_ptr_mut(&mut self) -> *mut T {
    self.data.as_mut_ptr().cast()
  }

  #[inline]
  const fn do_push(&mut self, value: T) -> Result<(), T> {
    let len = self.len;
    if len >= Self::N_U32 {
      return Err(value);
    }
    // SAFETY: `len` is within `N` bounds
    let dst = unsafe { self.data.as_mut_ptr().add(Usize::from_u32(len).into_usize()) };
    // SAFETY: `dst` points to valid uninitialized memory
    unsafe {
      ptr::write(dst, MaybeUninit::new(value));
    }
    self.len = len.wrapping_add(1);
    Ok(())
  }

  #[inline]
  const unsafe fn get_owned(&mut self, idx: u32) -> T {
    // SAFETY: It is up to the caller to provide a valid index
    let src = unsafe { self.data.as_ptr().add(Usize::from_u32(idx).into_usize()) };
    // SAFETY: If the index is valid, then the element exists
    let elem = unsafe { ptr::read(src) };
    // SAFETY: If the index is valid, then the element is initialized
    unsafe { elem.assume_init() }
  }
}

impl<T, const N: usize> ArrayVector<T, N>
where
  T: Clone,
{
  /// Creates a new instance with the copyable elements of `slice`.
  #[inline]
  pub fn from_cloneable_slice(slice: &[T]) -> crate::Result<Self> {
    let mut this = Self::new();
    this.extend_from_cloneable_slice(slice)?;
    Ok(this)
  }

  /// Iterates over the slice `other`, clones each element and then appends
  /// it to this vector. The `other` slice is traversed in-order.
  #[inline]
  pub fn extend_from_cloneable_slice(&mut self, other: &[T]) -> crate::Result<()> {
    for elem in other {
      self.push(elem.clone())?;
    }
    Ok(())
  }
}

impl<T, const N: usize> ArrayVector<T, N>
where
  T: Copy,
{
  /// Creates a new instance with the copyable elements of `slice`.
  #[inline]
  pub fn from_copyable_slice(slice: &[T]) -> crate::Result<Self> {
    let mut this = Self::new();
    this.extend_from_copyable_slice(slice)?;
    Ok(this)
  }

  /// Iterates over the slice `other`, copies each element and then appends
  /// it to this vector. The `other` slice is traversed in-order.
  #[inline]
  pub const fn extend_from_copyable_slice(&mut self, other: &[T]) -> crate::Result<()> {
    let len = self.len;
    let other_len_usize = other.len();
    let other_len_u32 = 'block: {
      if let Some(other_len_u32) = Usize::from_usize(other_len_usize).into_u32() {
        if other_len_u32 <= self.remaining() {
          break 'block other_len_u32;
        }
      }
      return Err(crate::Error::ArrayVectorError(ArrayVectorError::ExtendFromSliceOverflow));
    };
    // SAFETY: The above check ensures bounds
    let dst = unsafe { self.as_ptr_mut().add(Usize::from_u32(len).into_usize()) };
    // SAFETY: Parameters are valid
    unsafe {
      ptr::copy_nonoverlapping(other.as_ptr(), dst, other_len_usize);
    }
    self.len = len.wrapping_add(other_len_u32);
    Ok(())
  }
}

impl<T, const N: usize> Clone for ArrayVector<T, N>
where
  T: Clone,
{
  #[inline]
  fn clone(&self) -> Self {
    let mut this = Self::new();
    let _rslt = this.extend_from_cloneable_slice(self);
    this
  }
}

impl<T, const N: usize> Debug for ArrayVector<T, N>
where
  T: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    self.lease().fmt(f)
  }
}

impl<T, const N: usize> Default for ArrayVector<T, N> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<T, const N: usize> Deref for ArrayVector<T, N> {
  type Target = [T];

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.as_slice()
  }
}

impl<T, const N: usize> DerefMut for ArrayVector<T, N> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    // SAFETY: `len` ensures initialized elements
    unsafe { slice::from_raw_parts_mut(self.as_ptr_mut(), *Usize::from(self.len)) }
  }
}

impl<T, const N: usize> Drop for ArrayVector<T, N> {
  #[inline]
  fn drop(&mut self) {
    if Self::NEEDS_DROP {
      self.clear();
    }
  }
}

impl<T, const N: usize> Eq for ArrayVector<T, N> where T: Eq {}

impl<T, const N: usize> FromIterator<T> for Wrapper<crate::Result<ArrayVector<T, N>>> {
  #[inline]
  fn from_iter<I>(iter: I) -> Self
  where
    I: IntoIterator<Item = T>,
  {
    Wrapper(ArrayVector::from_iter(iter))
  }
}

impl<T, const N: usize> IntoIterator for ArrayVector<T, N> {
  type IntoIter = IntoIter<T, N>;
  type Item = T;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    IntoIter { idx: 0, data: self }
  }
}

impl<'any, T, const N: usize> IntoIterator for &'any ArrayVector<T, N>
where
  T: 'any,
{
  type IntoIter = slice::Iter<'any, T>;
  type Item = &'any T;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

impl<'any, T, const N: usize> IntoIterator for &'any mut ArrayVector<T, N>
where
  T: 'any,
{
  type IntoIter = slice::IterMut<'any, T>;
  type Item = &'any mut T;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.iter_mut()
  }
}

impl<T, const N: usize> Lease<[T]> for ArrayVector<T, N> {
  #[inline]
  fn lease(&self) -> &[T] {
    self
  }
}

impl<T, const N: usize> LeaseMut<[T]> for ArrayVector<T, N> {
  #[inline]
  fn lease_mut(&mut self) -> &mut [T] {
    self
  }
}

impl<T, const N: usize> PartialEq for ArrayVector<T, N>
where
  T: PartialEq,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

impl<T, const N: usize> PartialEq<[T]> for ArrayVector<T, N>
where
  T: PartialEq,
{
  #[inline]
  fn eq(&self, other: &[T]) -> bool {
    **self == *other
  }
}

impl<T, const N: usize> PartialOrd for ArrayVector<T, N>
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

impl<T, const N: usize> Ord for ArrayVector<T, N>
where
  T: Ord,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    (**self).cmp(other)
  }
}

impl<T, const N: usize> From<[T; N]> for ArrayVector<T, N> {
  #[inline]
  fn from(from: [T; N]) -> Self {
    Self::from_parts(from, None)
  }
}

impl<T, const N: usize> TryFrom<&[T]> for ArrayVector<T, N>
where
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

impl<const N: usize> fmt::Write for ArrayVector<u8, N> {
  #[inline]
  fn write_char(&mut self, c: char) -> fmt::Result {
    self.extend_from_copyable_slice(char_slice(&mut [0; 4], c)).map_err(|_err| fmt::Error)
  }

  #[inline]
  fn write_str(&mut self, s: &str) -> fmt::Result {
    self.extend_from_copyable_slice(s.as_bytes()).map_err(|_err| fmt::Error)
  }
}

#[cfg(feature = "std")]
impl<const N: usize> std::io::Write for ArrayVector<u8, N> {
  #[inline]
  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }

  #[inline]
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    let len = (*Usize::from(self.remaining())).min(buf.len());
    let _rslt = self.extend_from_copyable_slice(buf.get(..len).unwrap_or_default());
    Ok(len)
  }
}

/// A by-value array iterator.
#[derive(Debug)]
pub struct IntoIter<T, const N: usize> {
  idx: u32,
  data: ArrayVector<T, N>,
}

impl<T, const N: usize> IntoIter<T, N> {
  const NEEDS_DROP: bool = needs_drop::<T>();
}

impl<T, const N: usize> DoubleEndedIterator for IntoIter<T, N> {
  #[inline]
  fn next_back(&mut self) -> Option<Self::Item> {
    let Some(diff @ 1..=u32::MAX) = self.data.len.checked_sub(1) else {
      return None;
    };
    self.data.len = diff;
    // SAFETY: `diff` is within bounds
    Some(unsafe { self.data.get_owned(diff) })
  }
}

impl<T, const N: usize> Drop for IntoIter<T, N> {
  #[inline]
  fn drop(&mut self) {
    let idx = self.idx;
    let len = self.data.len;
    self.data.len = 0;
    if Self::NEEDS_DROP {
      let diff = len.wrapping_sub(idx);
      if diff > 0 {
        // SAFETY: Indices are within bounds
        unsafe {
          drop_elements(*Usize::from(diff), idx, self.data.as_ptr_mut());
        }
      }
    }
  }
}

impl<T, const N: usize> ExactSizeIterator for IntoIter<T, N> {}

impl<T, const N: usize> Iterator for IntoIter<T, N> {
  type Item = T;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.idx >= self.data.len {
      return None;
    }
    let idx = self.idx;
    self.idx = idx.wrapping_add(1);
    // SAFETY: `idx` is within bounds
    Some(unsafe { self.data.get_owned(idx) })
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = *Usize::from(self.data.len.wrapping_sub(self.idx));
    (len, Some(len))
  }
}

#[inline]
unsafe fn drop_elements<T>(len: usize, offset: u32, ptr: *mut T) {
  // SAFETY: It is up to the caller to provide a valid pointer with a valid index
  let data = unsafe { ptr.add(*Usize::from(offset)) };
  // SAFETY: It is up to the caller to provide a valid length
  let elements = unsafe { slice::from_raw_parts_mut(data, len) };
  // SAFETY: It is up to the caller to provide parameters that can lead to droppable elements
  unsafe {
    ptr::drop_in_place(elements);
  }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
  use crate::{collection::ArrayVector, misc::Usize};
  use arbitrary::{Arbitrary, Unstructured};

  impl<'any, T, const N: usize> Arbitrary<'any> for ArrayVector<T, N>
  where
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
  use crate::collection::ArrayVector;
  use core::{fmt::Formatter, marker::PhantomData};
  use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, SeqAccess, Visitor},
    ser::SerializeTuple,
  };

  impl<'de, T, const N: usize> Deserialize<'de> for ArrayVector<T, N>
  where
    T: Deserialize<'de>,
  {
    #[inline]
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
      DE: Deserializer<'de>,
    {
      struct ArrayVisitor<T, const N: usize>(PhantomData<T>);

      impl<'de, T, const N: usize> Visitor<'de> for ArrayVisitor<T, N>
      where
        T: Deserialize<'de>,
      {
        type Value = ArrayVector<T, N>;

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

      deserializer.deserialize_seq(ArrayVisitor::<T, N>(PhantomData))
    }
  }

  impl<T, const N: usize> Serialize for ArrayVector<T, N>
  where
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
