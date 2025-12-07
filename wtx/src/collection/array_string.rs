use crate::{
  collection::{
    LinearStorageLen,
    linear_storage::{
      LinearStorage, linear_storage_mut::LinearStorageMut, linear_storage_slice::LinearStorageSlice,
    },
  },
  misc::{Lease, from_utf8_basic},
};
use core::{
  borrow::Borrow,
  cmp::Ordering,
  fmt::{self, Arguments, Debug, Display, Formatter, Write},
  hash::{Hash, Hasher},
  ops::Deref,
  str,
};

/// [`ArrayString`] with a capacity limited by `u8`.
pub type ArrayStringU8<const N: usize> = ArrayString<u8, N>;
/// [`ArrayString`] with a capacity limited by `u16`.
pub type ArrayStringU16<const N: usize> = ArrayString<u16, N>;
/// [`ArrayString`] with a capacity limited by `u32`.
pub type ArrayStringU32<const N: usize> = ArrayString<u32, N>;
/// [`ArrayString`] with a capacity limited by `usize`.
pub type ArrayStringUsize<const N: usize> = ArrayString<usize, N>;

/// Errors of [`ArrayString`].
#[derive(Debug)]
pub enum ArrayStringError {
  /// Inner array is not fully
  IncompleteArray,
  #[doc = doc_reserve_overflow!()]
  ReserveOverflow,
}

/// A wrapper around the std's vector with some additional methods to manipulate copyable data.
#[derive(Clone, Copy)]
pub struct ArrayString<L, const N: usize>(Inner<L, N>);

impl<L, const N: usize> ArrayString<L, N>
where
  L: LinearStorageLen,
{
  const INSTANCE_CHECK: () = {
    assert!(N <= L::UPPER_BOUND_USIZE);
  };

  /// Constructs a new instance from a complete byte array.
  #[inline]
  pub fn from_array(data: [u8; N]) -> crate::Result<Self> {
    const { Self::INSTANCE_CHECK };
    Self::from_parts(data, L::UPPER_BOUND)
  }

  /// Constructs a new instance from a byte array and an explicit length.
  ///
  /// If `len` is greater than the capacity, then `len` will be truncated to `N`.
  #[inline]
  pub fn from_parts(data: [u8; N], len: L) -> crate::Result<Self> {
    const { Self::INSTANCE_CHECK };
    let instance_len = if len > L::UPPER_BOUND { L::UPPER_BOUND } else { len };
    let _ = from_utf8_basic(data.get(..instance_len.usize()).unwrap_or_default())?;
    // SAFETY: delimited data is UTF-8
    Ok(unsafe { Self::from_parts_unchecked(data, instance_len) })
  }

  /// Constructs a new, empty instance without verifying if the delimited bytes are valid UTF-8.
  ///
  /// If `len` is greater than the capacity, then `len` will be truncated to `N`.
  ///
  /// # Safety
  ///
  /// It is up to the caller to provide valid UTF-8 bytes until `len`.
  #[inline]
  pub unsafe fn from_parts_unchecked(data: [u8; N], len: L) -> Self {
    const { Self::INSTANCE_CHECK };
    Self(Inner { len: if len > L::UPPER_BOUND { L::UPPER_BOUND } else { len }, data })
  }

  /// Constructs a new, empty instance.
  #[inline]
  pub const fn new() -> Self {
    const { Self::INSTANCE_CHECK };
    Self(Inner { len: L::ZERO, data: [0; N] })
  }

  /// Constructs a new instance full of `NULL` characters.
  #[inline]
  pub const fn zeroed() -> Self {
    const { Self::INSTANCE_CHECK };
    Self(Inner { len: L::UPPER_BOUND, data: [0; N] })
  }

  /// The filled elements that composed a string.
  #[inline]
  pub fn as_str(&self) -> &str {
    self
  }

  /// The filled elements that composed a string.
  #[inline]
  pub fn data(&self) -> crate::Result<&[u8; N]> {
    if self.0.len.usize() != N {
      return Err(ArrayStringError::IncompleteArray.into());
    }
    Ok(&self.0.data)
  }
}

impl<L, const N: usize> ArrayString<L, N>
where
  L: LinearStorageLen,
{
  #[doc = from_iter_doc!("ArrayStringUsize::<16>", "\"123\".chars()", "\"123\"")]
  #[inline]
  pub fn from_iterator(iter: impl IntoIterator<Item = char>) -> crate::Result<Self> {
    Ok(Self(Inner::from_iterator(iter)?))
  }

  #[doc = as_slice_doc!("ArrayStringUsize::<16>", "\"123\".chars()", "\"123\"")]
  #[inline]
  pub fn as_slice(&self) -> &str {
    self.0.as_slice()
  }

  #[doc = as_slice_mut_doc!()]
  #[inline]
  pub fn as_slice_mut(&mut self) -> &mut str {
    self.0.as_slice_mut()
  }

  #[doc = capacity_doc!("ArrayStringUsize::<16>", "\"123\".chars()")]
  #[inline]
  pub fn capacity(&self) -> L {
    self.0.capacity()
  }

  #[doc = clear_doc!("ArrayStringUsize::<16>", "\"123\".chars()")]
  #[inline]
  pub fn clear(&mut self) {
    self.0.clear();
  }

  #[doc = extend_from_iter_doc!("ArrayStringUsize::<16>", "\"123\".chars()", "\"123\"")]
  #[inline]
  pub fn extend_from_iter(&mut self, iter: impl IntoIterator<Item = char>) -> crate::Result<()> {
    self.0.extend_from_iter(iter)
  }

  #[doc = len_doc!()]
  #[inline]
  pub fn len(&self) -> L {
    self.0.len()
  }

  #[doc = pop_doc!("ArrayStringUsize::<16>", "\"123\".chars()", "\"12\"")]
  #[inline]
  pub fn pop(&mut self) -> Option<char> {
    <str as LinearStorageSlice>::pop(&mut self.0)
  }

  #[doc = push_doc!("ArrayStringUsize::<16>", "'1'", "\"1\"")]
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

  #[doc = remaining_doc!("ArrayStringUsize::<16>", "'1'")]
  #[inline]
  pub fn remaining(&self) -> L {
    self.0.remaining()
  }

  #[doc = remove_doc!("ArrayStringUsize::<16>", "\"123\".chars()", "\"13\"")]
  #[inline]
  pub fn remove(&mut self, index: L) -> Option<char> {
    <str as LinearStorageSlice>::remove(&mut self.0, index)
  }

  #[doc = set_len_doc!()]
  #[inline]
  pub const unsafe fn set_len(&mut self, new_len: L) {
    self.0.len = new_len;
  }

  #[doc = truncate_doc!("ArrayStringUsize::<16>", "\"123\".chars()", "\"1\"")]
  #[inline]
  pub fn truncate(&mut self, new_len: L) {
    let _rslt = <str as LinearStorageSlice>::truncate(&mut self.0, new_len);
  }
}

impl<L, const N: usize> Borrow<str> for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn borrow(&self) -> &str {
    self
  }
}

impl<L, const N: usize> Debug for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(self)
  }
}

impl<L, const N: usize> Default for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<L, const N: usize> Display for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(self)
  }
}

impl<L, const N: usize> Deref for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  type Target = str;

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.0.as_slice()
  }
}

impl<L, const N: usize> Lease<str> for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn lease(&self) -> &str {
    self
  }
}

impl<L, const N: usize> Eq for ArrayString<L, N> where L: LinearStorageLen {}

impl<L, const N: usize> Hash for ArrayString<L, N>
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

impl<L, const N: usize> PartialEq for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

impl<L, const N: usize> PartialEq<[u8]> for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn eq(&self, other: &[u8]) -> bool {
    self.as_bytes() == other
  }
}

impl<L, const N: usize> PartialEq<str> for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn eq(&self, other: &str) -> bool {
    self.as_str() == other
  }
}

impl<L, const N: usize> PartialOrd for ArrayString<L, N>
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
  fn partial_cmp(&self, other: &Self) -> Option<Ordering>
  where
    L: LinearStorageLen,
  {
    Some(self.cmp(other))
  }
}

impl<L, const N: usize> Ord for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    (**self).cmp(other)
  }
}

impl<'args, L, const N: usize> TryFrom<Arguments<'args>> for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(from: Arguments<'args>) -> Result<Self, Self::Error> {
    let mut v = Self::new();
    v.write_fmt(from)?;
    Ok(v)
  }
}

impl<L, const N: usize> TryFrom<&[u8]> for ArrayString<L, N>
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

impl<L, const N: usize> TryFrom<&str> for ArrayString<L, N>
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

impl<L, const N: usize> Write for ArrayString<L, N>
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

#[derive(Clone, Copy)]
struct Inner<L, const N: usize> {
  len: L,
  data: [u8; N],
}

impl<L, const N: usize> Inner<L, N>
where
  L: LinearStorageLen,
{
  const INSTANCE_CHECK: () = {
    assert!(N <= L::UPPER_BOUND_USIZE);
  };
}

impl<L, const N: usize> LinearStorage<u8> for Inner<L, N>
where
  L: LinearStorageLen,
{
  type Len = L;
  type Slice = str;

  #[inline]
  fn as_ptr(&self) -> *const u8 {
    self.data.as_ptr().cast()
  }

  #[inline]
  fn capacity(&self) -> Self::Len {
    L::from_usize(N).unwrap_or_default()
  }

  #[inline]
  fn len(&self) -> Self::Len {
    self.len
  }
}

impl<L, const N: usize> LinearStorageMut<u8> for Inner<L, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn as_ptr_mut(&mut self) -> *mut u8 {
    self.data.as_mut_ptr().cast()
  }

  #[inline]
  fn reserve(&mut self, additional: Self::Len) -> crate::Result<()> {
    if additional > self.remaining() {
      return Err(ArrayStringError::ReserveOverflow.into());
    }
    Ok(())
  }

  fn reserve_exact(&mut self, additional: Self::Len) -> crate::Result<()> {
    if additional > self.remaining() {
      return Err(ArrayStringError::ReserveOverflow.into());
    }
    Ok(())
  }

  #[inline]
  unsafe fn set_len(&mut self, new_len: Self::Len) {
    self.len = new_len;
  }
}

impl<L, const N: usize> Default for Inner<L, N>
where
  L: LinearStorageLen,
{
  fn default() -> Self {
    const { Self::INSTANCE_CHECK };
    Self { len: L::ZERO, data: [0; N] }
  }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
  use crate::collection::{ArrayString, LinearStorageLen, array_string::Inner};
  use arbitrary::{Arbitrary, Unstructured};

  impl<'any, L, const N: usize> Arbitrary<'any> for ArrayString<L, N>
  where
    L: LinearStorageLen,
  {
    #[inline]
    fn arbitrary(u: &mut Unstructured<'any>) -> arbitrary::Result<Self> {
      let len = loop {
        let n = usize::arbitrary(u)?;
        if let Ok(converted) = L::from_usize(n) {
          break converted;
        }
      };
      let mut data = [0; N];
      for elem in data.iter_mut().take(len.usize()) {
        loop {
          let byte = u8::arbitrary(u)?;
          if byte.is_ascii_alphanumeric() {
            *elem = byte;
            break;
          }
        }
      }
      Ok(Self(Inner { len, data }))
    }
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::{
    collection::{ArrayString, LinearStorageLen},
    misc::from_utf8_basic,
  };
  use core::{fmt::Formatter, marker::PhantomData};
  use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Visitor},
  };

  impl<'de, L, const N: usize> Deserialize<'de> for ArrayString<L, N>
  where
    L: LinearStorageLen,
  {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      struct ArrayStringVisitor<L, const N: usize>(PhantomData<L>);

      impl<L, const N: usize> Visitor<'_> for ArrayStringVisitor<L, N>
      where
        L: LinearStorageLen,
      {
        type Value = ArrayString<L, N>;

        #[inline]
        fn expecting(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
          write!(formatter, "a string no more than {N} bytes long")
        }

        #[inline]
        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
          E: de::Error,
        {
          let rslt = from_utf8_basic(v);
          let str = rslt.map_err(|_| E::invalid_value(de::Unexpected::Bytes(v), &self))?;
          ArrayString::try_from(str).map_err(|_| E::invalid_length(str.len(), &self))
        }

        #[inline]
        fn visit_str<E>(self, str: &str) -> Result<Self::Value, E>
        where
          E: de::Error,
        {
          ArrayString::try_from(str).map_err(|_| E::invalid_length(str.len(), &self))
        }
      }

      deserializer.deserialize_str(ArrayStringVisitor(PhantomData))
    }
  }

  impl<L, const N: usize> Serialize for ArrayString<L, N>
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
