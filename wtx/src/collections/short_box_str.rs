use crate::{
  collections::{LinearStorageLen, ShortBoxSlice},
  misc::{Lease, from_utf8_basic},
};
use alloc::{boxed::Box, string::String};
use core::{
  fmt::{Debug, Formatter},
  ops::Deref,
  str,
};

/// [`ShortBoxStr`] with a capacity limited by `u8`.
pub type ShortBoxStrU8 = ShortBoxStr<u8>;
/// [`ShortBoxStr`] with a capacity limited by `u16`.
pub type ShortBoxStrU16 = ShortBoxStr<u16>;

/// An unaligned structure that has 9~10 bytes in `x86_64`. Useful in places where a bunch of
/// standard slices would take too much space.
#[derive(Default)]
pub struct ShortBoxStr<L>(ShortBoxSlice<L, u8>)
where
  L: LinearStorageLen;

impl<L> ShortBoxStr<L>
where
  L: LinearStorageLen,
{
  /// Throws an error if the length is greater than the chosen capacity.
  #[inline]
  pub fn new(data: Box<str>) -> crate::Result<Self> {
    Ok(Self(ShortBoxSlice::new(data.into_boxed_bytes())?))
  }

  /// Underlying byte slice
  #[inline]
  pub fn into_short_slice(self) -> ShortBoxSlice<L, u8> {
    self.0
  }

  /// Length
  #[inline]
  pub const fn len(&self) -> L {
    self.0.len()
  }
}

impl<L> Lease<str> for ShortBoxStr<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn lease(&self) -> &str {
    self
  }
}

impl<L> AsRef<str> for ShortBoxStr<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn as_ref(&self) -> &str {
    self
  }
}

impl<L> Clone for ShortBoxStr<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl<L> Debug for ShortBoxStr<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    (**self).fmt(f)
  }
}

impl<L> Deref for ShortBoxStr<L>
where
  L: LinearStorageLen,
{
  type Target = str;

  #[inline]
  fn deref(&self) -> &Self::Target {
    // SAFETY: Constructors only accept strings
    unsafe { str::from_utf8_unchecked(&self.0) }
  }
}

impl<L> PartialEq for ShortBoxStr<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl<L> TryFrom<&[u8]> for ShortBoxStr<L>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    String::from(from_utf8_basic(value)?).try_into()
  }
}

impl<L> TryFrom<&str> for ShortBoxStr<L>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    String::from(value).try_into()
  }
}

impl<L> TryFrom<Box<str>> for ShortBoxStr<L>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl<L> TryFrom<String> for ShortBoxStr<L>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::new(value.into())
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::collections::{LinearStorageLen, ShortBoxStr};
  use alloc::string::String;
  use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};

  impl<'de, L> Deserialize<'de> for ShortBoxStr<L>
  where
    L: LinearStorageLen,
  {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      let string = String::deserialize(deserializer)?;
      string.try_into().map_err(D::Error::custom)
    }
  }

  impl<L> Serialize for ShortBoxStr<L>
  where
    L: LinearStorageLen,
  {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      serializer.serialize_str(self)
    }
  }
}
