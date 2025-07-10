use crate::{
  collection::{IndexedStorage, IndexedStorageLen, IndexedStorageMut},
  misc::{Lease, from_utf8_basic},
};
use core::{
  borrow::Borrow,
  cmp::Ordering,
  fmt::{self, Arguments, Debug, Display, Formatter, Write},
  hash::{Hash, Hasher},
  ops::Deref,
  ptr, slice, str,
};

/// [`ArrayString`] with a capacity limited by `u8`.
pub type ArrayStringU8<const N: usize> = ArrayString<u8, N>;
/// [`ArrayString`] with a capacity limited by `u16`.
pub type ArrayStringU16<const N: usize> = ArrayString<u16, N>;
/// [`ArrayString`] with a capacity limited by `u32`.
pub type ArrayStringU32<const N: usize> = ArrayString<u32, N>;

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
pub struct ArrayString<L, const N: usize> {
  len: L,
  data: [u8; N],
}

impl<L, const N: usize> ArrayString<L, N>
where
  L: IndexedStorageLen,
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
    Self { len: if len > L::UPPER_BOUND { L::UPPER_BOUND } else { len }, data }
  }

  /// Constructs a new, empty instance.
  #[inline]
  pub const fn new() -> Self {
    const { Self::INSTANCE_CHECK };
    Self { len: L::ZERO, data: [0; N] }
  }

  /// Constructs a new instance full of `NULL` characters.
  #[inline]
  pub const fn zeroed() -> Self {
    const { Self::INSTANCE_CHECK };
    Self { len: L::UPPER_BOUND, data: [0; N] }
  }

  /// The filled elements that composed a string.
  #[inline]
  pub fn array(&self) -> crate::Result<&[u8; N]> {
    if self.len.usize() == N {
      return Ok(&self.data);
    }
    Err(ArrayStringError::IncompleteArray.into())
  }

  /// The filled elements that composed a string.
  #[inline]
  pub fn as_str(&self) -> &str {
    self
  }

  /// Alias of [`IndexedStorageMut::extend_from_copyable_slice`].
  #[inline]
  pub fn push_str(&mut self, str: &str) -> crate::Result<()> {
    self.extend_from_copyable_slice(str)
  }

  unsafe fn drop_elements(len: L, offset: L, ptr: *mut u8) {
    // SAFETY: it is up to the caller to provide a valid pointer with a valid index
    let data = unsafe { ptr.add(offset.usize()) };
    // SAFETY: it is up to the caller to provide a valid length
    let elements = unsafe { slice::from_raw_parts_mut(data, len.usize()) };
    // SAFETY: it is up to the caller to provide parameters that can lead to droppable elements
    unsafe {
      ptr::drop_in_place(elements);
    }
  }
}

impl<L, const N: usize> IndexedStorage<u8> for ArrayString<L, N>
where
  L: IndexedStorageLen,
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

impl<L, const N: usize> IndexedStorageMut<u8> for ArrayString<L, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn as_ptr_mut(&mut self) -> *mut u8 {
    self.data.as_mut_ptr().cast()
  }

  #[inline]
  fn pop(&mut self) -> Option<char> {
    let ch = self.as_slice().chars().next_back()?;
    self.len = self.len.wrapping_sub(Self::Len::from_usize(ch.len_utf8()).ok()?);
    Some(ch)
  }

  #[inline]
  fn reserve(&mut self, additional: Self::Len) -> crate::Result<()> {
    if additional > self.remaining() {
      return Err(ArrayStringError::ReserveOverflow.into());
    }
    Ok(())
  }

  #[inline]
  unsafe fn set_len(&mut self, new_len: Self::Len) {
    self.len = new_len;
  }

  #[inline]
  fn truncate(&mut self, new_len: Self::Len) {
    let len = self.len;
    let diff = if let Some(diff) = len.checked_sub(new_len)
      && diff > L::ZERO
    {
      diff
    } else {
      return;
    };
    self.len = new_len;
    if Self::NEEDS_DROP {
      // SAFETY: indices are within bounds
      unsafe {
        Self::drop_elements(diff, new_len, self.as_ptr_mut());
      }
    }
  }
}

impl<L, const N: usize> Borrow<str> for ArrayString<L, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn borrow(&self) -> &str {
    self
  }
}

impl<L, const N: usize> Debug for ArrayString<L, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(self)
  }
}

impl<L, const N: usize> Default for ArrayString<L, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<L, const N: usize> Display for ArrayString<L, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(self)
  }
}

impl<L, const N: usize> Deref for ArrayString<L, N>
where
  L: IndexedStorageLen,
{
  type Target = str;

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.as_slice()
  }
}

impl<L, const N: usize> Lease<str> for ArrayString<L, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn lease(&self) -> &str {
    self
  }
}

impl<L, const N: usize> Eq for ArrayString<L, N> where L: IndexedStorageLen {}

impl<L, const N: usize> Hash for ArrayString<L, N>
where
  L: IndexedStorageLen,
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
  L: IndexedStorageLen,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

impl<L, const N: usize> PartialEq<[u8]> for ArrayString<L, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn eq(&self, other: &[u8]) -> bool {
    self.as_bytes() == other
  }
}

impl<L, const N: usize> PartialEq<str> for ArrayString<L, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn eq(&self, other: &str) -> bool {
    self.as_str() == other
  }
}

impl<L, const N: usize> PartialOrd for ArrayString<L, N>
where
  L: IndexedStorageLen,
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
    L: IndexedStorageLen,
  {
    Some(self.cmp(other))
  }
}

impl<L, const N: usize> Ord for ArrayString<L, N>
where
  L: IndexedStorageLen,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    (**self).cmp(other)
  }
}

impl<'args, L, const N: usize> TryFrom<Arguments<'args>> for ArrayString<L, N>
where
  L: IndexedStorageLen,
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
  L: IndexedStorageLen,
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
  L: IndexedStorageLen,
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
  L: IndexedStorageLen,
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

#[cfg(feature = "arbitrary")]
mod arbitrary {
  use crate::collection::{ArrayString, IndexedStorageLen};
  use arbitrary::{Arbitrary, Unstructured};

  impl<'any, L, const N: usize> Arbitrary<'any> for ArrayString<L, N>
  where
    L: IndexedStorageLen,
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
      Ok(Self { len, data })
    }
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::{
    collection::{ArrayString, IndexedStorageLen},
    misc::from_utf8_basic,
  };
  use core::{fmt::Formatter, marker::PhantomData};
  use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Visitor},
  };

  impl<'de, L, const N: usize> Deserialize<'de> for ArrayString<L, N>
  where
    L: IndexedStorageLen,
  {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      struct ArrayStringVisitor<L, const N: usize>(PhantomData<L>);

      impl<L, const N: usize> Visitor<'_> for ArrayStringVisitor<L, N>
      where
        L: IndexedStorageLen,
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
    L: IndexedStorageLen,
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
