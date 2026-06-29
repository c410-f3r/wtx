use crate::{
  collections::{LinearStorageLen, ShortSlice},
  misc::{Lease, from_utf8_basic},
};
use core::{
  fmt::{Debug, Formatter},
  ops::Deref,
  str,
};

/// [`ShortStr`] with a capacity limited by `u8`.
pub type ShortStrU8<'any> = ShortStr<'any, u8>;
/// [`ShortStr`] with a capacity limited by `u16`.
pub type ShortStrU16<'any> = ShortStr<'any, u16>;

/// An unaligned structure that has 9~10 bytes in `x86_64`. Useful in places where a bunch of
/// standard slices would take too much space.
#[derive(Clone, Copy, Default)]
pub struct ShortStr<'any, L>(ShortSlice<'any, L, u8>)
where
  L: LinearStorageLen;

impl<'any, L> ShortStr<'any, L>
where
  L: LinearStorageLen,
{
  /// Throws an error if the length of `slice` is greater than the capacity.
  #[inline]
  pub fn new(slice: &'any str) -> crate::Result<Self> {
    Ok(Self(ShortSlice::new(slice.as_bytes())?))
  }

  /// Length
  #[inline]
  pub const fn len(self) -> L {
    self.0.len()
  }

  /// Underlying byte slice
  #[inline]
  pub fn into_short_slice(self) -> ShortSlice<'any, L, u8> {
    self.0
  }

  /// Owned method that returns the original string with its associated lifetime.
  #[inline]
  pub fn into_str(self) -> &'any str {
    // SAFETY: Constructors only accept strings
    unsafe { str::from_utf8_unchecked(self.0.into_slice()) }
  }
}

impl<'any> ShortStrU8<'any> {
  /// If necessary, `slice` is truncated to the maximum length capacity.
  #[inline]
  pub const fn new_truncated_u8(slice: &'any str) -> Self {
    Self(ShortSlice::new_truncated_u8(slice.as_bytes()))
  }
}

impl<L> Lease<str> for ShortStr<'_, L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn lease(&self) -> &str {
    self
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
    self.into_str()
  }
}

impl<L> Eq for ShortStr<'_, L> where L: LinearStorageLen {}

impl<'any, L> From<ShortStr<'any, L>> for &'any str
where
  L: LinearStorageLen,
{
  #[inline]
  fn from(value: ShortStr<'any, L>) -> Self {
    value.into_str()
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

impl<'any, L> TryFrom<&'any [u8]> for ShortStr<'any, L>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &'any [u8]) -> Result<Self, Self::Error> {
    Self::new(from_utf8_basic(value)?)
  }
}

impl<'any, L> TryFrom<&'any str> for ShortStr<'any, L>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &'any str) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}
