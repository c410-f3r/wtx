// Unaligned reads are UB if references are involved but in this scenario fields are copied
// by value.

use crate::collection::{LinearStorageLen, Vector};
use alloc::boxed::Box;
use core::{
  fmt::{Debug, Formatter},
  ops::Deref,
  ptr::NonNull,
  slice,
};

/// [`ShortBoxVector`] with a capacity limited by `u8`.
pub type ShortBoxVectorU8<T> = ShortBoxVector<u8, T>;
/// [`ShortBoxVector`] with a capacity limited by `u16`.
pub type ShortBoxVectorU16<T> = ShortBoxVector<u16, T>;

/// An unaligned structure that has 9~10 bytes in `x86_64`. Useful in places where a bunch of
/// standard slices would take too much space.
#[expect(clippy::repr_packed_without_abi, reason = "only used internally")]
#[repr(packed)]
pub struct ShortBoxVector<L, T>
where
  L: LinearStorageLen,
{
  ptr: NonNull<T>,
  len: L,
}

impl<L, T> ShortBoxVector<L, T>
where
  L: LinearStorageLen,
{
  const CHECK_LEN: () = { assert!(L::BYTES <= 2) };

  /// Throws an error if the length is greater than the chosen capacity.
  #[inline]
  pub fn new(data: Box<[T]>) -> crate::Result<Self> {
    const { Self::CHECK_LEN }
    let len = L::from_usize(data.len())?;
    // SAFETY: `Box` is guaranteed to be non-null.
    let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(data).cast()) };
    Ok(Self { ptr, len })
  }
}

impl<L, T> AsRef<[T]> for ShortBoxVector<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn as_ref(&self) -> &[T] {
    self
  }
}

impl<L, T> Debug for ShortBoxVector<L, T>
where
  L: LinearStorageLen,
  T: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    (**self).fmt(f)
  }
}

impl<L, T> Default for ShortBoxVector<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn default() -> Self {
    Self { ptr: NonNull::dangling(), len: L::ZERO }
  }
}

impl<L, T> Deref for ShortBoxVector<L, T>
where
  L: LinearStorageLen,
{
  type Target = [T];

  #[inline]
  fn deref(&self) -> &Self::Target {
    let ptr = self.ptr;
    let len = self.len;
    // SAFETY: pointer and length come from an allocated slice
    unsafe { slice::from_raw_parts(ptr.as_ptr(), len.usize()) }
  }
}

impl<L, T> Drop for ShortBoxVector<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn drop(&mut self) {
    let ptr = self.ptr;
    let len = self.len;
    if len == L::ZERO {
      return;
    }
    // SAFETY: pointer and length come from an allocated slice
    let slice = unsafe { slice::from_raw_parts_mut(ptr.as_ptr(), len.usize()) };
    // SAFETY: the instance was constructed from a valid `Box`.
    drop(unsafe { Box::from_raw(slice) });
  }
}

impl<L, T> PartialEq for ShortBoxVector<L, T>
where
  L: LinearStorageLen,
  T: PartialEq,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

impl<L, T> TryFrom<Box<[T]>> for ShortBoxVector<L, T>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: Box<[T]>) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl<L, T> TryFrom<Vector<T>> for ShortBoxVector<L, T>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: Vector<T>) -> Result<Self, Self::Error> {
    Self::new(value.into_vec().into())
  }
}

// SAFETY: pointer is not used to perform mutable operations behaving like a slice
unsafe impl<L, T: Send> Send for ShortBoxVector<L, T> where L: LinearStorageLen {}
// SAFETY: pointer is not used to perform mutable operations behaving like a slice
unsafe impl<L, T: Sync> Sync for ShortBoxVector<L, T> where L: LinearStorageLen {}

#[cfg(test)]
mod tests {
  use crate::collection::ShortBoxVectorU8;
  use alloc::boxed::Box;

  #[test]
  fn new() {
    let data = [1, 2];
    let instance = ShortBoxVectorU8::new(Box::new(data)).unwrap();
    assert_eq!(&*instance, data);
  }
}
