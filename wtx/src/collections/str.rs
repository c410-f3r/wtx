use crate::{
  collections::{
    LinearStorageLen, Vector,
    linear_storage::{
      LinearStorage, linear_storage_mut::LinearStorageMut, linear_storage_slice::LinearStorageSlice,
    },
  },
  misc::{Lease, LeaseMut, from_utf8_basic},
};
use alloc::string::String;
use core::{
  borrow::Borrow,
  cmp::Ordering,
  fmt::{self, Arguments, Debug, Display, Formatter, Write},
  hash::{Hash, Hasher},
  ops::{Deref, DerefMut},
  str,
};

/// [`Str`] with a capacity limited by `u8`.
pub type StrU8 = Str<u8>;
/// [`Str`] with a capacity limited by `u16`.
pub type StrU16 = Str<u16>;
/// [`Str`] with a capacity limited by `u32`.
pub type StrU32 = Str<u32>;
/// [`Str`] with a capacity limited by `usize`.
pub type StrUsize = Str<usize>;

/// Same as [`alloc::string::String`] with the difference that lengths can be arbitrary integers.
#[derive(Clone, Default)]
pub struct Str<L>(Inner<L>)
where
  L: LinearStorageLen;

impl<L> Str<L>
where
  L: LinearStorageLen,
{
  /// Constructs a new, empty instance.
  #[inline]
  pub const fn new() -> Self {
    Self(Inner::new())
  }
}

impl<L> Str<L>
where
  L: LinearStorageLen,
{
  #[doc = from_iter_doc!("StrUsize", "\"123\".chars()", "\"123\"")]
  #[inline]
  pub fn from_iterator(iter: impl IntoIterator<Item = char>) -> crate::Result<Self> {
    Ok(Self(Inner::from_iterator(iter)?))
  }

  #[doc = as_slice_doc!("StrUsize", "\"123\".chars()", "\"123\"")]
  #[inline]
  pub fn as_slice(&self) -> &str {
    self.0.as_slice()
  }

  #[doc = as_slice_mut_doc!()]
  #[inline]
  pub fn as_slice_mut(&mut self) -> &mut str {
    self.0.as_slice_mut()
  }

  #[doc = as_str_doc!("StrUsize", "\"123\".chars()", "\"123\"")]
  #[inline]
  pub fn as_str(&self) -> &str {
    self.0.as_slice()
  }

  #[doc = as_str_mut_doc!()]
  #[inline]
  pub fn as_str_mut(&mut self) -> &mut str {
    self.0.as_slice_mut()
  }

  #[doc = capacity_doc!("StrUsize", "\"123\".chars()")]
  #[inline]
  pub fn capacity(&self) -> L {
    self.0.capacity()
  }

  #[doc = clear_doc!("StrUsize", "\"123\".chars()")]
  #[inline]
  pub fn clear(&mut self) {
    self.0.clear();
  }

  #[doc = extend_from_iter_doc!("StrUsize", "\"123\".chars()", "\"123\"")]
  #[inline]
  pub fn extend_from_iter(&mut self, iter: impl IntoIterator<Item = char>) -> crate::Result<()> {
    self.0.extend_from_iter(iter)
  }

  #[doc = len_doc!()]
  #[inline]
  pub fn len(&self) -> L {
    self.0.len()
  }

  #[doc = pop_doc!("StrUsize", "\"123\".chars()", "\"12\"")]
  #[inline]
  pub fn pop(&mut self) -> Option<char> {
    <str as LinearStorageSlice>::pop(&mut self.0)
  }

  #[doc = push_doc!("StrUsize", "'1'", "\"1\"")]
  #[inline]
  pub fn push(&mut self, elem: char) -> crate::Result<()> {
    self.0.push(elem)
  }

  /// Appends a given string slice onto the end of this instance.
  #[inline]
  pub fn push_str(&mut self, other: &str) -> crate::Result<()> {
    self.0.extend_from_copyable_slice(other)
  }

  /// Appends a set of string slices onto the end of this instance.
  #[inline]
  pub fn push_strs<E, I>(&mut self, others: I) -> crate::Result<L>
  where
    E: Lease<str>,
    I: IntoIterator<Item = E>,
    I::IntoIter: Clone,
  {
    self.0.extend_from_copyable_slices(others)
  }

  #[doc = remaining_capacity_doc!("StrUsize", "'1'")]
  #[inline]
  pub fn remaining_capacity(&self) -> L {
    self.0.remaining_capacity()
  }

  #[doc = remove_doc!("StrUsize", "\"123\".chars()", "\"13\"")]
  #[inline]
  pub fn remove(&mut self, index: L) -> Option<char> {
    <str as LinearStorageSlice>::remove(&mut self.0, index)
  }

  #[doc = set_len_doc!()]
  #[inline]
  pub unsafe fn set_len(&mut self, new_len: L) {
    // SAFETY: Up to the caller
    unsafe { self.0.set_len(new_len) }
  }

  #[doc = truncate_doc!("StrUsize", "\"123\".chars()", "\"1\"")]
  #[inline]
  pub fn truncate(&mut self, new_len: L) {
    let _rslt = <str as LinearStorageSlice>::truncate(&mut self.0, new_len);
  }
}

impl<L> Borrow<str> for Str<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn borrow(&self) -> &str {
    self
  }
}

impl<L> Debug for Str<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(self)
  }
}

impl<L> Display for Str<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(self)
  }
}

impl<L> Deref for Str<L>
where
  L: LinearStorageLen,
{
  type Target = str;

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.0.as_slice()
  }
}

impl<L> DerefMut for Str<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.0.as_slice_mut()
  }
}

impl<L> Lease<[u8]> for Str<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn lease(&self) -> &[u8] {
    self.as_bytes()
  }
}

impl<L> Lease<str> for Str<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn lease(&self) -> &str {
    self
  }
}

impl<L> LeaseMut<str> for Str<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn lease_mut(&mut self) -> &mut str {
    self
  }
}

impl<L> Eq for Str<L> where L: LinearStorageLen {}

impl<L> Hash for Str<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn hash<H>(&self, state: &mut H)
  where
    H: Hasher,
  {
    Hash::hash(&**self, state);
  }
}

impl<L> PartialEq for Str<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

impl<L> PartialEq<String> for Str<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn eq(&self, other: &String) -> bool {
    self.as_str() == *other
  }
}

impl<L> PartialEq<[u8]> for Str<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn eq(&self, other: &[u8]) -> bool {
    self.as_bytes() == other
  }
}
impl<L> PartialEq<&[u8]> for Str<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn eq(&self, other: &&[u8]) -> bool {
    self.as_bytes() == *other
  }
}

impl<L> PartialEq<str> for Str<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn eq(&self, other: &str) -> bool {
    self.as_str() == other
  }
}
impl<L> PartialEq<&str> for Str<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn eq(&self, other: &&str) -> bool {
    self.as_str() == *other
  }
}

impl<L> PartialOrd for Str<L>
where
  L: LinearStorageLen,
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

impl<L> Ord for Str<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    (**self).cmp(&**other)
  }
}

impl<'args, L> TryFrom<Arguments<'args>> for Str<L>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(from: Arguments<'args>) -> Result<Self, Self::Error> {
    let mut rslt = Self::new();
    rslt.write_fmt(from)?;
    Ok(rslt)
  }
}

impl<L> TryFrom<&[u8]> for Str<L>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &[u8]) -> Result<Self, Self::Error> {
    let mut this = Self::default();
    this.push_str(from_utf8_basic(from)?)?;
    Ok(this)
  }
}

impl<L> TryFrom<&str> for Str<L>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &str) -> Result<Self, Self::Error> {
    let mut this = Self::default();
    this.push_str(from)?;
    Ok(this)
  }
}

impl<L> Write for Str<L>
where
  L: LinearStorageLen,
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

#[derive(Clone, Default)]
struct Inner<L>(Vector<L, u8>)
where
  L: LinearStorageLen;

impl<L> Inner<L>
where
  L: LinearStorageLen,
{
  #[inline]
  const fn new() -> Self {
    Self(Vector::new())
  }
}

impl<L> LinearStorage<u8> for Inner<L>
where
  L: LinearStorageLen,
{
  type Len = L;
  type Slice = str;

  #[inline]
  fn as_ptr(&self) -> *const u8 {
    self.0.as_ptr().cast()
  }

  #[inline]
  fn capacity(&self) -> Self::Len {
    self.0.capacity()
  }

  #[inline]
  fn len(&self) -> Self::Len {
    self.0.len()
  }
}

impl<L> LinearStorageMut<u8> for Inner<L>
where
  L: LinearStorageLen,
{
  #[inline]
  fn as_ptr_mut(&mut self) -> *mut u8 {
    self.0.as_ptr_mut()
  }

  #[inline]
  fn reserve(&mut self, additional: Self::Len) -> crate::Result<()> {
    self.0.reserve(additional)
  }

  fn reserve_exact(&mut self, additional: Self::Len) -> crate::Result<()> {
    self.0.reserve_exact(additional)
  }

  #[inline]
  unsafe fn set_len(&mut self, new_len: Self::Len) {
    // SAFETY: Up to the caller
    unsafe { self.0.set_len(new_len) }
  }
}

// SAFETY: there is no immutable method internally operating mutable modifications
unsafe impl<L> Send for Inner<L> where L: LinearStorageLen {}
// SAFETY: there is no immutable method internally operating mutable modifications
unsafe impl<L> Sync for Inner<L> where L: LinearStorageLen {}

#[cfg(feature = "serde")]
mod serde {
  use crate::{
    collections::{LinearStorageLen, Str},
    misc::from_utf8_basic,
  };
  use core::{fmt::Formatter, marker::PhantomData};
  use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Visitor},
  };

  impl<'de, L> Deserialize<'de> for Str<L>
  where
    L: LinearStorageLen,
  {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      struct StrVisitor<L>(PhantomData<L>);

      impl<L> Visitor<'_> for StrVisitor<L>
      where
        L: LinearStorageLen,
      {
        type Value = Str<L>;

        #[inline]
        fn expecting(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
          write!(formatter, "a string")
        }

        #[inline]
        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
          E: de::Error,
        {
          let rslt = from_utf8_basic(v);
          let str = rslt.map_err(|_err| E::invalid_value(de::Unexpected::Bytes(v), &self))?;
          Str::try_from(str).map_err(|_err| E::invalid_length(str.len(), &self))
        }

        #[inline]
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
          E: de::Error,
        {
          Str::try_from(v).map_err(|_err| E::invalid_length(v.len(), &self))
        }
      }

      deserializer.deserialize_str(StrVisitor(PhantomData))
    }
  }

  impl<L> Serialize for Str<L>
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
