use crate::misc::{Lease, from_utf8_basic};
use core::{
  borrow::Borrow,
  fmt::{self, Debug, Display, Formatter},
  ops::Deref,
  str::{self, FromStr},
};

/// Errors of [`FixedStr`].
#[derive(Clone, Copy, Debug)]
pub enum FixedStrError {
  /// Bytes are not UTF-8
  NotUtf8,
  /// The length of the slice does not match the length of the array
  LengthMismatch,
}

/// A wrapper around a fixed array that is always UTF-8.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct FixedStr<const N: usize>([u8; N]);

impl<const N: usize> FixedStr<N> {
  /// New instance that verifies if `data` is UTF-8 and has the same length of `N`.
  #[inline]
  pub fn from_bytes(data: &[u8]) -> crate::Result<Self> {
    if data.len() != N {
      return Err(FixedStrError::LengthMismatch.into());
    }
    if from_utf8_basic(data).is_err() {
      return Err(FixedStrError::NotUtf8.into());
    }
    Ok(Self(data.try_into()?))
  }

  /// New instance that verifies if `data` is UTF-8
  #[inline]
  pub fn from_bytes_array(data: [u8; N]) -> crate::Result<Self> {
    if from_utf8_basic(&data).is_err() {
      return Err(FixedStrError::NotUtf8.into());
    }
    Ok(Self(data))
  }

  /// Constant version of [`Self::from_bytes_array`].
  #[inline]
  #[expect(clippy::disallowed_methods, reason = "the method of simutf8 is not constant")]
  pub const fn from_bytes_array_opt(data: [u8; N]) -> Option<Self> {
    if str::from_utf8(&data).is_err() {
      return None;
    }
    Some(Self(data))
  }

  /// New instance that does not verify if `data` is UTF-8
  ///
  /// # Safety
  ///
  /// `data` must be UTF-8
  #[inline]
  pub const unsafe fn from_bytes_array_unchecked(data: [u8; N]) -> Self {
    Self(data)
  }

  /// The inner bytes reinterpreted as a string
  #[inline]
  pub const fn as_str(&self) -> &str {
    // SAFETY: All constructors verify if the provided bytes are UTF-8
    unsafe { str::from_utf8_unchecked(&self.0) }
  }

  /// The inner bytes reinterpreted as a mutable string
  #[inline]
  pub const fn as_str_mut(&mut self) -> &mut str {
    // SAFETY: All constructors verify if the provided bytes are UTF-8
    unsafe { str::from_utf8_unchecked_mut(&mut self.0) }
  }

  /// Returns the inner array
  #[inline]
  pub const fn into_inner(self) -> [u8; N] {
    self.0
  }
}

impl<const N: usize> Borrow<str> for FixedStr<N> {
  #[inline]
  fn borrow(&self) -> &str {
    self
  }
}

impl<const N: usize> Debug for FixedStr<N> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(self)
  }
}

impl<const N: usize> Default for FixedStr<N> {
  #[inline]
  fn default() -> Self {
    Self([0; N])
  }
}

impl<const N: usize> Display for FixedStr<N> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(self)
  }
}

impl<const N: usize> Deref for FixedStr<N> {
  type Target = str;

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.as_str()
  }
}

impl<const N: usize> Lease<str> for FixedStr<N> {
  #[inline]
  fn lease(&self) -> &str {
    self
  }
}

impl<const N: usize> TryFrom<[u8; N]> for FixedStr<N> {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: [u8; N]) -> Result<Self, Self::Error> {
    Self::from_bytes_array(from)
  }
}

impl<const N: usize> TryFrom<&[u8]> for FixedStr<N> {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &[u8]) -> Result<Self, Self::Error> {
    Self::from_bytes(from)
  }
}

impl<const N: usize> TryFrom<&str> for FixedStr<N> {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &str) -> Result<Self, Self::Error> {
    if from.len() != N {
      return Err(FixedStrError::LengthMismatch.into());
    }
    Ok(Self(from.as_bytes().try_into()?))
  }
}

impl<const N: usize> FromStr for FixedStr<N> {
  type Err = crate::Error;

  #[inline]
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::try_from(s)
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::collections::FixedStr;
  use core::fmt::Formatter;
  use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Visitor},
  };

  impl<'de, const N: usize> Deserialize<'de> for FixedStr<N> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      struct LocalVisitor<const N: usize>;

      impl<const N: usize> Visitor<'_> for LocalVisitor<N> {
        type Value = FixedStr<N>;

        #[inline]
        fn expecting(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
          write!(formatter, "a string with exactly {N} bytes")
        }

        #[inline]
        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
          E: de::Error,
        {
          FixedStr::try_from(v).map_err(|_err| E::invalid_length(v.len(), &self))
        }

        #[inline]
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
          E: de::Error,
        {
          FixedStr::try_from(v).map_err(|_err| E::invalid_length(v.len(), &self))
        }
      }

      deserializer.deserialize_str(LocalVisitor)
    }
  }

  impl<const N: usize> Serialize for FixedStr<N> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      serializer.serialize_str(self)
    }
  }
}
