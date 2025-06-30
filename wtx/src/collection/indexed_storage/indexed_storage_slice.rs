use crate::misc::char_slice;
use core::slice;

/// Unsized slices like `str` or `[T]`.
pub trait IndexedStorageSlice {
  /// Underlying slice type
  type Data;
  /// The logical unit that this slice represents. Can be [`IndexedStorageSlice::Data`] or a set
  /// of [`IndexedStorageSlice::Data`]s.
  type Unit;

  /// Forms a slice from a pointer and a length.
  ///
  /// The `len` argument is the number of **elements**, not the number of bytes.
  ///
  /// # Safety
  ///
  /// The same safety rules of [`core::slice::from_raw_parts`] apply to this method.
  unsafe fn from_raw_parts<'any>(data: *const Self::Data, len: usize) -> &'any Self;

  /// Performs the same functionality as [`IndexedStorageSlice::from_raw_parts`], except that a
  /// mutable slice is returned.
  ///
  /// # Safety
  ///
  /// The same safety rules of [`core::slice::from_raw_parts_mut`] apply to this method.
  unsafe fn from_raw_parts_mut<'any>(data: *mut Self::Data, len: usize) -> &'any mut Self;

  /// Returns a reference to the underlying slice of [`IndexedStorageSlice::Data`] elements.
  fn data(&self) -> &[Self::Data];

  /// Returns one or more [`IndexedStorageSlice::Data`] that compose an [`IndexedStorageSlice::Unit`].
  fn data_from_unit(unit: Self::Unit) -> impl ExactSizeIterator<Item = Self::Data>;

  /// Returns an iterator over the logical units contained in this slice.
  fn units(&self) -> impl Iterator<Item = Self::Unit>
  where
    Self::Unit: Clone;
}

impl IndexedStorageSlice for str {
  type Data = u8;
  type Unit = char;

  #[inline]
  unsafe fn from_raw_parts<'any>(data: *const Self::Data, len: usize) -> &'any Self {
    // SAFETY: it is up to the caller to pass an initialized slice
    let slice = unsafe { slice::from_raw_parts(data, len) };
    // SAFETY: it is up to the caller to pass a valid UTF-8 slice
    unsafe { str::from_utf8_unchecked(slice) }
  }

  #[inline]
  unsafe fn from_raw_parts_mut<'any>(data: *mut Self::Data, len: usize) -> &'any mut Self {
    // SAFETY: it is up to the caller to pass an initialized slice
    let slice = unsafe { slice::from_raw_parts_mut(data, len) };
    // SAFETY: it is up to the caller to pass a valid UTF-8 slice
    unsafe { str::from_utf8_unchecked_mut(slice) }
  }

  #[inline]
  fn data(&self) -> &[Self::Data] {
    self.as_bytes()
  }

  #[inline]
  fn data_from_unit(unit: Self::Unit) -> impl ExactSizeIterator<Item = Self::Data> {
    let mut buffer = [0; 4];
    let len = char_slice(&mut buffer, unit).len();
    buffer.into_iter().take(len)
  }

  #[inline]
  fn units(&self) -> impl Iterator<Item = Self::Unit>
  where
    Self::Unit: Clone,
  {
    self.chars()
  }
}

impl<T> IndexedStorageSlice for [T] {
  type Data = T;
  type Unit = T;

  #[inline]
  unsafe fn from_raw_parts<'any>(data: *const Self::Data, len: usize) -> &'any Self {
    // SAFETY: it is up to the caller to pass an initialized slice
    unsafe { slice::from_raw_parts(data, len) }
  }

  #[inline]
  unsafe fn from_raw_parts_mut<'any>(data: *mut Self::Data, len: usize) -> &'any mut Self {
    // SAFETY: it is up to the caller to pass an initialized slice
    unsafe { slice::from_raw_parts_mut(data, len) }
  }

  #[inline]
  fn data(&self) -> &[Self::Data] {
    self
  }

  #[inline]
  fn data_from_unit(unit: Self::Unit) -> impl ExactSizeIterator<Item = Self::Data> {
    [unit].into_iter()
  }

  #[inline]
  fn units(&self) -> impl Iterator<Item = Self::Unit>
  where
    Self::Unit: Clone,
  {
    self.iter().cloned()
  }
}
