use crate::misc::{char_slice, Usize};
use core::{
  cmp::Ordering,
  fmt::{self, Arguments, Debug, Display, Formatter, Write},
  ops::Deref,
  str,
};

/// A wrapper around the std's vector with some additional methods to manipulate copyable data.
#[derive(Clone, Copy)]
pub struct ArrayString<const N: usize> {
  len: u32,
  data: [u8; N],
}

impl<const N: usize> ArrayString<N> {
  /// Constructs a new, empty instance.
  #[inline]
  pub const fn new() -> Self {
    if N > Usize::from_u32(u32::MAX).into_usize() {
      panic!("Capacity is too large");
    }
    Self { len: 0, data: [0; N] }
  }

  /// The filled elements that composed a string.
  #[inline]
  pub fn as_str(&self) -> &str {
    self
  }

  /// The number of elements that can be stored.
  #[inline]
  pub fn capacity(&self) -> u32 {
    N as _
  }

  /// Clears the vector, removing all values.
  #[inline]
  pub fn clear(&mut self) {
    self.len = 0;
  }

  /// How many elements can be added to this collection.
  #[inline]
  pub fn remaining(&self) -> u32 {
    self.capacity().wrapping_sub(self.len)
  }

  /// How many elements can be added to this collection.
  #[inline]
  pub fn replace(&mut self, start: usize, str: &str) -> crate::Result<()> {
    let Some(slice) = start.checked_add(str.len()).and_then(|end| self.data.get_mut(start..end))
    else {
      return Err(crate::Error::OutOfBoundsArithmetic);
    };
    slice.copy_from_slice(str.as_bytes());
    Ok(())
  }

  /// Appends an element to the back of the collection.
  #[inline]
  pub fn try_push(&mut self, cf: char) -> crate::Result<()> {
    self.try_push_bytes(char_slice(&mut [0; 4], cf))
  }

  /// Iterates over the slice `other`, copies each element, and then appends
  /// it to this vector. The `other` slice is traversed in-order.
  ///
  /// # Panics
  ///
  /// If there is no available capacity.
  #[inline]
  pub fn try_push_str(&mut self, str: &str) -> crate::Result<()> {
    self.try_push_bytes(str.as_bytes())
  }

  /// Shortens the vector, keeping the first `len` elements.
  #[inline]
  pub fn truncate(&mut self, len: u32) {
    self.len = len.min(self.capacity());
  }

  #[inline]
  fn try_push_bytes(&mut self, other: &[u8]) -> crate::Result<()> {
    let Some(len) = u32::try_from(other.len()).ok().filter(|el| self.remaining() >= *el) else {
      return Err(crate::Error::CapacityOverflow);
    };
    let begin = *Usize::from(self.len);
    let end = *Usize::from(self.len.wrapping_add(len));
    self.data.get_mut(begin..end).unwrap_or_default().copy_from_slice(other);
    self.len = self.len.wrapping_add(len);
    Ok(())
  }
}

impl<const N: usize> AsRef<str> for ArrayString<N> {
  #[inline]
  fn as_ref(&self) -> &str {
    self
  }
}

impl<const N: usize> Debug for ArrayString<N> {
  #[inline]
  fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
    f.write_str(self.as_str())
  }
}

impl<const N: usize> Default for ArrayString<N> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<const N: usize> Display for ArrayString<N> {
  #[inline]
  fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
    f.write_str(self.as_str())
  }
}

impl<const N: usize> Deref for ArrayString<N> {
  type Target = str;

  #[inline]
  fn deref(&self) -> &Self::Target {
    unsafe { str::from_utf8_unchecked(self.data.get(..*Usize::from(self.len)).unwrap_or_default()) }
  }
}

impl<const N: usize> Eq for ArrayString<N> {}

impl<const N: usize> PartialEq for ArrayString<N> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

impl<const N: usize> PartialEq<[u8]> for ArrayString<N> {
  #[inline]
  fn eq(&self, other: &[u8]) -> bool {
    self.as_bytes() == other
  }
}

impl<const N: usize> PartialEq<str> for ArrayString<N> {
  #[inline]
  fn eq(&self, other: &str) -> bool {
    self.as_str() == other
  }
}

impl<const N: usize> PartialOrd for ArrayString<N> {
  #[inline]
  fn ge(&self, other: &Self) -> bool {
    (&**self).ge(&**other)
  }

  #[inline]
  fn gt(&self, other: &Self) -> bool {
    (&**self).gt(&**other)
  }

  #[inline]
  fn le(&self, other: &Self) -> bool {
    (&**self).le(&**other)
  }

  #[inline]
  fn lt(&self, other: &Self) -> bool {
    (&**self).lt(&**other)
  }

  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    (&**self).partial_cmp(&**other)
  }
}

impl<const N: usize> Ord for ArrayString<N> {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    (**self).cmp(other)
  }
}

impl<'args, const N: usize> TryFrom<Arguments<'args>> for ArrayString<N> {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: Arguments<'args>) -> Result<Self, Self::Error> {
    let mut v = Self::new();
    v.write_fmt(from)?;
    Ok(v)
  }
}

impl<const N: usize> TryFrom<&str> for ArrayString<N> {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &str) -> Result<Self, Self::Error> {
    let mut this = Self::default();
    this.try_push_str(from)?;
    Ok(this)
  }
}

impl<const N: usize> Write for ArrayString<N> {
  #[inline]
  fn write_char(&mut self, ch: char) -> fmt::Result {
    self.try_push(ch).map_err(|_| fmt::Error)
  }

  #[inline]
  fn write_str(&mut self, str: &str) -> fmt::Result {
    self.try_push_str(str).map_err(|_| fmt::Error)
  }
}
