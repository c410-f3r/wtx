use crate::collection::{LinearStorageLen, ShortSlice};
use core::{
  fmt::{Debug, Formatter},
  ops::Deref,
  str,
};

/// [`ShortSlice`] with a capacity limited by `u8`.
pub type ShortStrU8<'any> = ShortStr<'any, u8>;
/// [`ShortSlice`] with a capacity limited by `u16`.
pub type ShortStrU16<'any> = ShortStr<'any, u16>;

/// An unaligned structure that has 9~10 bytes in `x86_64`. Useful in places where a bunch of
/// standard slices would take too much space.
#[derive(Clone, Copy, Default)]
pub struct ShortStr<'any, L>(ShortSlice<'any, L, u8>)
where
  L: LinearStorageLen;

impl<'any> ShortStrU8<'any> {
  /// If necessary, `slice` is truncated to the maximum length capacity.
  #[inline]
  pub const fn new_truncated_u8(slice: &'any str) -> Self {
    Self(ShortSlice::new_truncated_u8(slice.as_bytes()))
  }
}

impl<L> AsRef<str> for ShortStr<'_, L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn as_ref(&self) -> &str {
    self
  }
}

impl<L> Debug for ShortStr<'_, L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    (**self).fmt(f)
  }
}

impl<L> Deref for ShortStr<'_, L>
where
  L: LinearStorageLen,
{
  type Target = str;

  #[inline]
  fn deref(&self) -> &Self::Target {
    // SAFETY: Constructors only accept strings
    unsafe { str::from_utf8_unchecked(self.0.data()) }
  }
}

impl<'any> From<&'any str> for ShortStrU8<'any> {
  #[inline]
  fn from(value: &'any str) -> Self {
    Self::new_truncated_u8(value)
  }
}

impl<L> PartialEq for ShortStr<'_, L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}
