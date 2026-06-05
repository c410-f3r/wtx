// Unaligned reads are UB if references are involved but in this scenario fields are copied
// by value.

use crate::collection::{LinearStorageLen, Vector};
use alloc::{boxed::Box, vec::Vec};
use core::{
  fmt::{Debug, Formatter},
  mem::ManuallyDrop,
  ops::Deref,
  ptr::{self, NonNull},
  slice,
};

/// [`ShortBoxSlice`] with a capacity limited by `u8`.
pub type ShortBoxSliceU8<T> = ShortBoxSlice<u8, T>;
/// [`ShortBoxSlice`] with a capacity limited by `u16`.
pub type ShortBoxSliceU16<T> = ShortBoxSlice<u16, T>;

/// An unaligned structure that has 9~10 bytes in `x86_64`. Useful in places where a bunch of
/// standard slices would take too much space.
#[expect(clippy::repr_packed_without_abi, reason = "only used internally")]
#[repr(packed)]
pub struct ShortBoxSlice<L, T>
where
  L: LinearStorageLen,
{
  ptr: NonNull<T>,
  len: L,
}

impl<L, T> ShortBoxSlice<L, T>
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

  /// Length
  #[inline]
  pub const fn len(&self) -> L {
    self.len
  }
}

impl<L, T> AsRef<[T]> for ShortBoxSlice<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn as_ref(&self) -> &[T] {
    self
  }
}

impl<L, T> Clone for ShortBoxSlice<L, T>
where
  L: LinearStorageLen,
  T: Clone,
{
  #[inline]
  fn clone(&self) -> Self {
    let slice = &**self;
    // SAFETY: Length is already verified
    unsafe { slice.to_vec().try_into().unwrap_unchecked() }
  }
}

impl<L, T> Debug for ShortBoxSlice<L, T>
where
  L: LinearStorageLen,
  T: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    (**self).fmt(f)
  }
}

impl<L, T> Default for ShortBoxSlice<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn default() -> Self {
    Self { ptr: NonNull::dangling(), len: L::ZERO }
  }
}

impl<L, T> Deref for ShortBoxSlice<L, T>
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

impl<L, T> Drop for ShortBoxSlice<L, T>
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
    let slice = ptr::slice_from_raw_parts_mut(ptr.as_ptr(), len.usize());
    // SAFETY: the instance was constructed from a valid `Box`.
    drop(unsafe { Box::from_raw(slice) });
  }
}

impl<L, T> PartialEq for ShortBoxSlice<L, T>
where
  L: LinearStorageLen,
  T: PartialEq,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

impl<L, T> From<ShortBoxSlice<L, T>> for Box<[T]>
where
  L: LinearStorageLen,
{
  #[inline]
  fn from(value: ShortBoxSlice<L, T>) -> Self {
    Vec::<T>::from(value).into_boxed_slice()
  }
}

impl<L, T> From<ShortBoxSlice<L, T>> for Vec<T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn from(value: ShortBoxSlice<L, T>) -> Self {
    let instance = ManuallyDrop::new(value);
    let len = instance.len;
    let ptr = instance.ptr;
    // SAFETY: pointer and length came from a valid box where the capacity equals the length
    unsafe { Vec::from_raw_parts(ptr.as_ptr(), len.usize(), len.usize()) }
  }
}

impl<L, T> From<ShortBoxSlice<L, T>> for Vector<T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn from(value: ShortBoxSlice<L, T>) -> Self {
    Vector::from_vec(Vec::<T>::from(value))
  }
}

impl<L, T> TryFrom<&[T]> for ShortBoxSlice<L, T>
where
  L: LinearStorageLen,
  T: Clone,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &[T]) -> Result<Self, Self::Error> {
    Vec::<T>::from(value).try_into()
  }
}

impl<L, T> TryFrom<Box<[T]>> for ShortBoxSlice<L, T>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: Box<[T]>) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl<L, T> TryFrom<Vec<T>> for ShortBoxSlice<L, T>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
    Self::new(value.into())
  }
}

impl<L, T> TryFrom<Vector<T>> for ShortBoxSlice<L, T>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: Vector<T>) -> Result<Self, Self::Error> {
    value.into_vec().try_into()
  }
}

// SAFETY: pointer is not used to perform mutable operations behaving like a slice
unsafe impl<L, T: Send> Send for ShortBoxSlice<L, T> where L: LinearStorageLen {}
// SAFETY: pointer is not used to perform mutable operations behaving like a slice
unsafe impl<L, T: Sync> Sync for ShortBoxSlice<L, T> where L: LinearStorageLen {}

#[cfg(test)]
mod tests {
  use crate::collection::ShortBoxSliceU8;
  use alloc::boxed::Box;

  #[test]
  fn new() {
    let data = [1, 2];
    let instance = ShortBoxSliceU8::new(Box::new(data)).unwrap();
    assert_eq!(&*instance, data);
  }
}
