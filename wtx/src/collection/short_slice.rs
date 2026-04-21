use crate::collection::LinearStorageLen;
use core::{
  fmt::{Debug, Formatter},
  marker::PhantomData,
  ops::Deref,
  ptr, slice,
};

/// [`ShortSlice`] with a capacity limited by `u8`.
pub type ShortSliceU8<'any, T> = ShortSlice<'any, u8, T>;
/// [`ShortSlice`] with a capacity limited by `u16`.
pub type ShortSliceU16<'any, T> = ShortSlice<'any, u16, T>;

/// An unaligned structure that has 9~10 bytes in `x86_64`. Useful in places where a bunch of
/// standard slices would take too much space.
#[expect(clippy::repr_packed_without_abi, reason = "only used internally")]
#[repr(packed)]
pub struct ShortSlice<'any, L, T> {
  ptr: *const T,
  len: L,
  phantom: PhantomData<&'any T>,
}

impl<L, T> ShortSlice<'_, L, T>
where
  L: LinearStorageLen,
{
  const CHECK_LEN: () = { assert!(L::BYTES <= 2) };

  /// Reinterprets inner data back into a standard slice.
  #[inline]
  pub fn data(&self) -> &[T] {
    // SAFETY: Pointer and length come from a slice that is tied to `'any`
    unsafe { slice::from_raw_parts(self.ptr(), self.len().usize()) }
  }

  fn len(&self) -> L {
    // SAFETY: Pointer and length come from a slice that is tied to `'any`
    unsafe { ptr::addr_of!(self.len).read_unaligned() }
  }

  fn ptr(&self) -> *const T {
    // SAFETY: Pointer and length come from a slice that is tied to `'any`
    unsafe { ptr::addr_of!(self.ptr).read_unaligned() }
  }
}

impl<'any, T> ShortSliceU8<'any, T> {
  /// If necessary, `slice` is truncated to the maximum length capacity.
  #[inline]
  #[expect(clippy::cast_possible_truncation, reason = "lack of const support")]
  pub const fn new_truncated_u8(slice: &'any [T]) -> Self {
    const { Self::CHECK_LEN }
    Self {
      len: if slice.len() <= 255 { slice.len() as u8 } else { 255 },
      phantom: PhantomData,
      ptr: slice.as_ptr(),
    }
  }
}

impl<L, T> AsRef<[T]> for ShortSlice<'_, L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn as_ref(&self) -> &[T] {
    self
  }
}

impl<L, T> Clone for ShortSlice<'_, L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn clone(&self) -> Self {
    *self
  }
}

impl<L, T> Copy for ShortSlice<'_, L, T> where L: LinearStorageLen {}

impl<L, T> Debug for ShortSlice<'_, L, T>
where
  L: LinearStorageLen,
  T: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    (**self).fmt(f)
  }
}

impl<L, T> Default for ShortSlice<'_, L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn default() -> Self {
    Self { ptr: ptr::null(), len: L::ZERO, phantom: PhantomData }
  }
}

impl<L, T> Deref for ShortSlice<'_, L, T>
where
  L: LinearStorageLen,
{
  type Target = [T];

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.data()
  }
}

impl<'any, T> From<&'any [T]> for ShortSliceU8<'any, T> {
  #[inline]
  fn from(value: &'any [T]) -> Self {
    Self::new_truncated_u8(value)
  }
}

impl<L, T> PartialEq for ShortSlice<'_, L, T>
where
  L: LinearStorageLen,
  T: PartialEq,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

// SAFETY: Pointer is not used to perform mutable operations behaving like  &'any [T]
unsafe impl<L, T: Sync> Send for ShortSlice<'_, L, T> {}
// SAFETY: Pointer is not used to perform mutable operations behaving like  &'any [T]
unsafe impl<L, T: Sync> Sync for ShortSlice<'_, L, T> {}

#[cfg(test)]
mod tests {
  use crate::collection::ShortSliceU8;
  use core::mem;

  #[test]
  fn empty_slice() {
    let empty: &[u8] = &[];
    let short = ShortSliceU8::new_truncated_u8(empty);
    assert!(short.is_empty());
  }

  #[test]
  fn new_truncated() {
    let large_slice = &[0u8; 300];
    let short = ShortSliceU8::new_truncated_u8(large_slice);
    assert_eq!(short.len(), 255);
  }

  #[test]
  fn size_of() {
    assert!(mem::size_of::<ShortSliceU8<'_, u8>>() < 2usize * mem::size_of::<usize>());
  }

  #[test]
  fn size_of_in_context() {
    struct Foo<'any> {
      _a: ShortSliceU8<'any, u8>,
      _b: u32,
    }
    assert!(mem::size_of::<Foo<'_>>() <= 2usize * mem::size_of::<usize>());
  }

  #[test]
  fn static_instance() {
    static FOO: ShortSliceU8<'static, u8> = ShortSliceU8::new_truncated_u8(&[1, 2, 3]);
    assert_eq!(&*FOO, &[1, 2, 3]);
  }
}
