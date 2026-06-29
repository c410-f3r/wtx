use core::{mem::MaybeUninit, slice};

/// A slice of elements that may or may not contain an uninitialized part
#[derive(Debug)]
pub struct MaybeUninitSlice<'any, T> {
  initialized: usize,
  bytes: &'any mut [MaybeUninit<T>],
}

impl<'any, T> MaybeUninitSlice<'any, T> {
  /// Creates a new instance from a fully initialized buffer.
  #[inline]
  pub const fn from_initialized(bytes: &'any mut [T]) -> Self {
    let len = bytes.len();
    let ptr = bytes.as_mut_ptr();
    Self {
      initialized: len,
      // SAFETY: Just a simple conversion of an already initialized slice
      bytes: unsafe { slice::from_raw_parts_mut(ptr.cast(), len) },
    }
  }

  /// Creates a new instance from a slice that may be uninitialized.
  ///
  /// Use [`Self::assume_initialized`] to increase the amount of initialized elements.
  #[inline]
  pub const fn from_uninitialized(bytes: &'any mut [MaybeUninit<T>]) -> Self {
    Self { initialized: 0, bytes }
  }

  /// Returns a reference to the entire slice containing initialized and uninitialized data.
  #[inline]
  pub const fn all(&self) -> &[MaybeUninit<T>] {
    self.bytes
  }

  /// Mutable version of [`Self::all`].
  #[inline]
  pub const fn all_mut(&mut self) -> &mut [MaybeUninit<T>] {
    self.bytes
  }

  /// Assumes that the first `len` elements of the buffer are initialized. If `len` is greater
  /// than the slice's capacity, then `len` will be capped.
  ///
  /// Please note take this method can decrease the number of **LOGICALLY** initialized elements
  /// because the inner components are overwritten.
  ///
  /// # Safety
  ///
  /// The caller must ensure that the first `len` elements are indeed initialized.
  #[inline]
  pub const unsafe fn assume_initialized(&mut self, len: usize) {
    let bytes_len = self.bytes.len();
    self.initialized = if len > bytes_len { bytes_len } else { len };
  }

  /// Returns a mutable reference to the uninitialized part of the slice.
  #[inline]
  pub const fn uninitialized_mut(&mut self) -> &mut [MaybeUninit<T>] {
    let bytes = &mut *self.bytes;
    let initialized = self.initialized;
    // SAFETY: All constructors ensure that `initialized` really points to initialized data
    unsafe { bytes.split_at_mut_unchecked(initialized).1 }
  }
}

#[allow(single_use_lifetimes, reason = "depends on feature")]
impl<'any> MaybeUninitSlice<'any, u8> {
  /// Fills the uninitialized bytes returning everything.
  #[inline]
  pub fn initialize_all_bytes(&mut self) -> &mut [u8] {
    self.uninitialized_mut().fill(MaybeUninit::new(0));
    self.initialized = self.bytes.len();
    // SAFETY: Everything has just been initialized
    unsafe { self.bytes.assume_init_mut() }
  }

  #[cfg(feature = "tokio")]
  #[inline]
  pub(crate) fn into_tokio_read_buf(self) -> tokio::io::ReadBuf<'any> {
    let bytes = self.bytes;
    let initialized = self.initialized;
    let mut rslt = tokio::io::ReadBuf::uninit(bytes);
    // SAFETY: All constructors ensure that `initialized` really points to initialized data
    unsafe {
      rslt.assume_init(initialized);
    }
    rslt
  }
}

impl<'any, T> From<&'any mut [T]> for MaybeUninitSlice<'any, T> {
  #[inline]
  fn from(value: &'any mut [T]) -> Self {
    Self::from_initialized(value)
  }
}

impl<'any, T> From<&'any mut [MaybeUninit<T>]> for MaybeUninitSlice<'any, T> {
  #[inline]
  fn from(value: &'any mut [MaybeUninit<T>]) -> Self {
    Self::from_uninitialized(value)
  }
}
