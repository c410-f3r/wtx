use crate::collection::{LinearStorageLen, ShortBoxVector};
use alloc::string::String;
use core::{
  fmt::{Debug, Formatter},
  ops::Deref,
  str,
};

/// [`ShortBoxString`] with a capacity limited by `u8`.
pub type ShortBoxStringU8 = ShortBoxString<u8>;
/// [`ShortBoxString`] with a capacity limited by `u16`.
pub type ShortBoxStringU16 = ShortBoxString<u16>;

/// An unaligned structure that has 9~10 bytes in `x86_64`. Useful in places where a bunch of
/// standard slices would take too much space.
#[derive(Default)]
pub struct ShortBoxString<L>(ShortBoxVector<L, u8>)
where
  L: LinearStorageLen;

impl<L> ShortBoxString<L>
where
  L: LinearStorageLen,
{
  /// Throws an error if the length is greater than the chosen capacity.
  #[inline]
  pub fn new(data: String) -> crate::Result<Self> {
    Ok(Self(ShortBoxVector::new(data.into_bytes().into())?))
  }
}

impl<L> AsRef<str> for ShortBoxString<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn as_ref(&self) -> &str {
    self
  }
}

impl<L> Debug for ShortBoxString<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    (**self).fmt(f)
  }
}

impl<L> Deref for ShortBoxString<L>
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

impl<L> PartialEq for ShortBoxString<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl<L> TryFrom<String> for ShortBoxString<L>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::collection::{LinearStorageLen, ShortBoxString};
  use alloc::string::String;
  use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

  impl<'de, L> Deserialize<'de> for ShortBoxString<L>
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

  impl<L> Serialize for ShortBoxString<L>
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
