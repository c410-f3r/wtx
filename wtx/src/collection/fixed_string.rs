use crate::misc::{Lease, from_utf8_basic};
use core::{
  borrow::Borrow,
  fmt::{self, Debug, Display, Formatter},
  ops::Deref,
  str,
};

/// Errors of [`FixedString`].
#[derive(Debug)]
pub enum FixedStringError {
  /// Bytes are not UTF-8
  NotUtf8,
  /// The length of the slice does not match the length of the array
  LengthMismatch,
}

/// A wrapper around a fixed array that is always UTF-8.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct FixedString<const N: usize>([u8; N]);

impl<const N: usize> FixedString<N> {
  /// New instance that verifies if `data` is UTF-8
  #[inline]
  pub fn from_array(data: [u8; N]) -> crate::Result<Self> {
    if from_utf8_basic(&data).is_err() {
      return Err(FixedStringError::NotUtf8.into());
    }
    Ok(Self(data))
  }

  /// New instance that does not verify if `data` is UTF-8
  ///
  /// # Safety
  ///
  /// `data` must be UTF-8
  #[inline]
  pub const unsafe fn from_array_unchecked(data: [u8; N]) -> Self {
    Self(data)
  }

  /// New instance that verifies if `data` is UTF-8 and has the same length of `N`.
  #[inline]
  pub fn from_slice(data: &[u8]) -> crate::Result<Self> {
    if data.len() != N {
      return Err(FixedStringError::LengthMismatch.into());
    }
    if from_utf8_basic(data).is_err() {
      return Err(FixedStringError::NotUtf8.into());
    }
    Ok(Self(data.try_into()?))
  }

  /// The inner bytes reinterpreted as a string
  #[inline]
  pub const fn as_str(&self) -> &str {
    // SAFETY: All constructors verify if the provided bytes are UTF-8
    unsafe { str::from_utf8_unchecked(&self.0) }
  }

  /// Returns the inner array
  #[inline]
  pub const fn into_inner(self) -> [u8; N] {
    self.0
  }
}

impl<const N: usize> Borrow<str> for FixedString<N> {
  #[inline]
  fn borrow(&self) -> &str {
    self
  }
}

impl<const N: usize> Debug for FixedString<N> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(self)
  }
}

impl<const N: usize> Default for FixedString<N> {
  #[inline]
  fn default() -> Self {
    Self([0; N])
  }
}

impl<const N: usize> Display for FixedString<N> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(self)
  }
}

impl<const N: usize> Deref for FixedString<N> {
  type Target = str;

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.as_str()
  }
}

impl<const N: usize> Lease<str> for FixedString<N> {
  #[inline]
  fn lease(&self) -> &str {
    self
  }
}

impl<const N: usize> TryFrom<[u8; N]> for FixedString<N> {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: [u8; N]) -> Result<Self, Self::Error> {
    Self::from_array(from)
  }
}

impl<const N: usize> TryFrom<&[u8]> for FixedString<N> {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &[u8]) -> Result<Self, Self::Error> {
    Self::from_slice(from)
  }
}

impl<const N: usize> TryFrom<&str> for FixedString<N> {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &str) -> Result<Self, Self::Error> {
    if from.len() != N {
      return Err(FixedStringError::LengthMismatch.into());
    }
    Ok(Self(from.as_bytes().try_into()?))
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::collection::FixedString;
  use core::fmt::Formatter;
  use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Visitor},
  };

  impl<'de, const N: usize> Deserialize<'de> for FixedString<N> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      struct LocalVisitor<const N: usize>;

      impl<const N: usize> Visitor<'_> for LocalVisitor<N> {
        type Value = FixedString<N>;

        #[inline]
        fn expecting(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
          write!(formatter, "a string with exactly {N} bytes")
        }

        #[inline]
        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
          E: de::Error,
        {
          FixedString::try_from(v).map_err(|_err| E::invalid_length(v.len(), &self))
        }

        #[inline]
        fn visit_str<E>(self, str: &str) -> Result<Self::Value, E>
        where
          E: de::Error,
        {
          FixedString::try_from(str).map_err(|_err| E::invalid_length(str.len(), &self))
        }
      }

      deserializer.deserialize_str(LocalVisitor)
    }
  }

  impl<const N: usize> Serialize for FixedString<N> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      serializer.serialize_str(self)
    }
  }
}
