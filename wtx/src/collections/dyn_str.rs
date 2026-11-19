use crate::{
  collections::{
    DynVector, SparseLinearStorageLen, Truncate, U7Len, U15Len, U31Len, misc::is_char_boundary,
  },
  misc::{Lease, LeaseMut, char_slice, from_utf8_basic},
};
use alloc::string::String;
use core::{
  borrow::Borrow,
  cmp::Ordering,
  fmt::{self, Arguments, Debug, Display, Formatter, Write},
  hash::{Hash, Hasher},
  ops::{Deref, DerefMut},
  str::{self, FromStr},
};

/// [`DynStr`] with a capacity limited by `u7`.
pub type DynStrU7<const N: usize> = DynStr<U7Len, N>;
/// [`DynStr`] with a capacity limited by `u15`.
pub type DynStrU15<const N: usize> = DynStr<U15Len, N>;
/// [`DynStr`] with a capacity limited by `u31`.
pub type DynStrU31<const N: usize> = DynStr<U31Len, N>;

/// A `String`-like container that can store a small number of elements in the stack.
pub struct DynStr<L, const N: usize>(DynVector<L, u8, N>)
where
  L: SparseLinearStorageLen;

impl<L, const N: usize> DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[doc = from_iter_doc!("DynStrU7::<16>", "\"123\".chars()", "\"123\"")]
  #[inline]
  pub fn from_iterator(iter: impl IntoIterator<Item = char>) -> crate::Result<Self> {
    let mut this = Self::new();
    for elem in iter {
      this.push(elem)?;
    }
    Ok(this)
  }

  /// Constructs a new, empty instance.
  ///
  /// ```rust
  /// use wtx::collections::LinearStorageLen;
  /// let string = wtx::collections::DynStrU7::<16>::new();
  /// assert_eq!(string.len().usize(), 0);
  /// ```
  #[inline]
  pub const fn new() -> Self {
    Self(DynVector::new())
  }

  /// Constructs a new, empty instance with at least the specified capacity.
  #[inline]
  pub fn with_capacity(capacity: L::Storage) -> crate::Result<Self> {
    Ok(Self(DynVector::with_capacity(capacity)?))
  }

  /// Returns the contents of this instance as a byte slice.
  #[inline]
  pub fn as_bytes(&self) -> &[u8] {
    self.0.as_slice()
  }

  #[doc = as_slice_doc!("DynStrU7::<16>", "\"123\".chars()", "\"123\"")]
  #[inline]
  pub fn as_slice(&self) -> &str {
    self.as_str()
  }

  #[doc = as_slice_mut_doc!("DynStrU7::<16>", "\"123\".chars()", "\"123\"")]
  #[inline]
  pub fn as_slice_mut(&mut self) -> &mut str {
    self.as_str_mut()
  }

  #[doc = as_str_doc!("DynStrU7::<16>", "\"123\".chars()", "\"123\"")]
  #[inline]
  pub fn as_str(&self) -> &str {
    // SAFETY: All constructors and mutating methods preserve valid UTF-8
    unsafe { str::from_utf8_unchecked(self.0.as_slice()) }
  }

  #[doc = as_str_mut_doc!("DynStrU7::<16>")]
  #[inline]
  pub fn as_str_mut(&mut self) -> &mut str {
    // SAFETY: All constructors and mutating methods preserve valid UTF-8
    unsafe { str::from_utf8_unchecked_mut(self.0.as_slice_mut()) }
  }

  #[doc = capacity_doc!("DynStrU7::<16>", "\"123\".chars()")]
  #[inline]
  pub fn capacity(&self) -> L {
    self.0.capacity()
  }

  #[doc = clear_doc!("DynStrU7::<16>", "\"123\".chars()")]
  #[inline]
  pub fn clear(&mut self) {
    self.0.clear();
  }

  #[doc = len_doc!("DynStrU7::<16>", "\"123\".chars()")]
  #[inline]
  pub fn len(&self) -> L {
    self.0.len()
  }

  #[doc = push_doc!("DynStrU7::<16>", "'1'", "\"1\"")]
  #[inline]
  pub fn push(&mut self, elem: char) -> crate::Result<()> {
    self.0.extend_from_copyable_slice(char_slice(&mut [0; 4], elem).as_bytes())
  }

  /// Appends a given string slice onto the end of this instance.
  #[inline]
  pub fn push_str(&mut self, other: &str) -> crate::Result<()> {
    self.0.extend_from_copyable_slice(other.as_bytes())
  }

  #[doc = remaining_capacity_doc!("DynStrU7::<16>", "'1'")]
  #[inline]
  pub fn remaining_capacity(&self) -> L {
    self.0.remaining_capacity()
  }

  #[doc = remaining_capacity_max_doc!("DynStrU7::<16>", "\"123\".chars()", "wtx::collections::U7Len::MAX.get() as usize - 3")]
  #[inline]
  pub fn remaining_capacity_max(&self) -> L {
    self.0.remaining_capacity_max()
  }

  #[doc = reserve_doc!("DynStrU7::<16>")]
  #[inline]
  pub fn reserve(&mut self, additional: L::Storage) -> crate::Result<()> {
    self.0.reserve(additional)
  }

  #[doc = reserve_exact_doc!("DynStrU7::<16>")]
  #[inline]
  pub fn reserve_exact(&mut self, additional: L::Storage) -> crate::Result<()> {
    self.0.reserve_exact(additional)
  }
}

impl<L, const N: usize> AsRef<str> for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn as_ref(&self) -> &str {
    self
  }
}

impl<L, const N: usize> Borrow<str> for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn borrow(&self) -> &str {
    self
  }
}

impl<L, const N: usize> Clone for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl<L, const N: usize> Debug for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(self)
  }
}

impl<L, const N: usize> Default for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<L, const N: usize> Deref for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  type Target = str;

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.as_str()
  }
}

impl<L, const N: usize> DerefMut for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.as_str_mut()
  }
}

impl<L, const N: usize> Display for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(self)
  }
}

impl<L, const N: usize> Eq for DynStr<L, N> where L: SparseLinearStorageLen {}

impl<L, const N: usize> FromStr for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  type Err = crate::Error;

  #[inline]
  fn from_str(value: &str) -> Result<Self, Self::Err> {
    value.try_into()
  }
}

impl<L, const N: usize> Hash for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn hash<H>(&self, state: &mut H)
  where
    H: Hasher,
  {
    Hash::hash(&**self, state);
  }
}

impl<L, const N: usize> Lease<[u8]> for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn lease(&self) -> &[u8] {
    self.as_bytes()
  }
}

impl<L, const N: usize> Lease<str> for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn lease(&self) -> &str {
    self
  }
}

impl<L, const N: usize> LeaseMut<str> for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn lease_mut(&mut self) -> &mut str {
    self
  }
}

impl<L, const N: usize> PartialEq for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

impl<L, const N: usize> PartialEq<String> for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn eq(&self, other: &String) -> bool {
    self.as_str() == *other
  }
}

impl<L, const N: usize> PartialEq<[u8]> for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn eq(&self, other: &[u8]) -> bool {
    self.as_bytes() == other
  }
}

impl<L, const N: usize> PartialEq<&[u8]> for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn eq(&self, other: &&[u8]) -> bool {
    self.as_bytes() == *other
  }
}

impl<L, const N: usize> PartialEq<str> for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn eq(&self, other: &str) -> bool {
    self.as_str() == other
  }
}

impl<L, const N: usize> PartialEq<&str> for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn eq(&self, other: &&str) -> bool {
    self.as_str() == *other
  }
}

impl<L, const N: usize> PartialOrd for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn ge(&self, other: &Self) -> bool {
    (**self).ge(&**other)
  }

  #[inline]
  fn gt(&self, other: &Self) -> bool {
    (**self).gt(&**other)
  }

  #[inline]
  fn le(&self, other: &Self) -> bool {
    (**self).le(&**other)
  }

  #[inline]
  fn lt(&self, other: &Self) -> bool {
    (**self).lt(&**other)
  }

  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl<L, const N: usize> Ord for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    (**self).cmp(&**other)
  }
}

impl<L, const N: usize> Truncate for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn truncate(&mut self, input: usize) {
    let new_len = L::from_usize(input).unwrap_or(L::UPPER_BOUND);
    if !is_char_boundary(new_len.usize(), self.0.as_slice()) {
      return;
    }
    Truncate::truncate(&mut self.0, input);
  }
}

impl<'args, L, const N: usize> TryFrom<Arguments<'args>> for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(from: Arguments<'args>) -> Result<Self, Self::Error> {
    let mut rslt = Self::new();
    rslt.write_fmt(from)?;
    Ok(rslt)
  }
}

impl<L, const N: usize> TryFrom<&[u8]> for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &[u8]) -> Result<Self, Self::Error> {
    let mut this = Self::new();
    this.push_str(from_utf8_basic(from)?)?;
    Ok(this)
  }
}

impl<L, const N: usize> TryFrom<&str> for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &str) -> Result<Self, Self::Error> {
    let mut this = Self::new();
    this.push_str(from)?;
    Ok(this)
  }
}

impl<L, const N: usize> Write for DynStr<L, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn write_char(&mut self, c: char) -> fmt::Result {
    self.push(c).map_err(|_err| fmt::Error)
  }

  #[inline]
  fn write_str(&mut self, s: &str) -> fmt::Result {
    self.push_str(s).map_err(|_err| fmt::Error)
  }
}
