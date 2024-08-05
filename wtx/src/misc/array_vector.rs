use crate::misc::{char_slice, Lease, LeaseMut, Usize};
use core::{
  array,
  cmp::Ordering,
  fmt::{self, Debug, Formatter},
  iter,
  mem::{needs_drop, MaybeUninit},
  ops::{Deref, DerefMut},
  ptr, slice,
};

/// Errors of [`ArrayVector`].
#[derive(Debug)]
pub enum ArrayVectorError {
  #[doc = doc_many_elems_cap_overflow!()]
  ExtendFromSliceOverflow,
  /// Inner array is not totally full
  IntoInnerIncomplete,
  #[doc = doc_single_elem_cap_overflow!()]
  PushOverflow,
}

/// A wrapper around the std's vector with some additional methods to manipulate copyable data.
pub struct ArrayVector<D, const N: usize> {
  len: u32,
  data: [MaybeUninit<D>; N],
}

impl<D, const N: usize> ArrayVector<D, N> {
  /// Constructs a new instance reusing any `data` elements delimited by `len`.
  #[inline]
  pub fn from_array<const M: usize>(array: [D; M]) -> Self {
    const {
      assert!(M <= N);
    }
    let mut this = Self::new();
    for elem in array {
      let _rslt = this.push(elem);
    }
    this
  }

  /// Constructs a new instance reusing any `data` elements delimited by `len`.
  #[expect(clippy::should_implement_trait, reason = "The std trait ins infallible")]
  #[inline]
  pub fn from_iter(iter: impl IntoIterator<Item = D>) -> Result<Self, ArrayVectorError> {
    let mut this = Self::new();
    for elem in iter.into_iter().take(N) {
      this.push(elem)?;
    }
    Ok(this)
  }

  /// Constructs a new instance reusing any `data` elements delimited by `len`.
  #[expect(clippy::needless_pass_by_value, reason = "false positive")]
  #[inline]
  pub fn from_parts(data: [D; N], len: u32) -> Self {
    Self::instance_check();
    let n = Self::instance_u32();
    Self {
      len: if len > n { n } else { len },
      // SAFETY: `data` is fully initialized within `u32` bounds and `D` does not implement `Drop`
      data: unsafe { ptr::from_ref::<[D; N]>(&data).cast::<[MaybeUninit<D>; N]>().read() },
    }
  }

  /// Constructs a new empty instance.
  #[inline]
  pub const fn new() -> Self {
    Self::instance_check();
    Self { len: 0, data: [const { MaybeUninit::uninit() }; N] }
  }

  /// Extracts a slice containing the entire vector.
  #[inline]
  pub fn as_slice(&self) -> &[D] {
    self
  }

  /// The number of elements that can be stored.
  #[inline]
  pub fn capacity(&self) -> u32 {
    const {
      let [_, _, _, _, a, b, c, d] = Usize::from_usize(N).into_u64().to_be_bytes();
      u32::from_be_bytes([a, b, c, d])
    }
  }

  /// Clears the vector, removing all values.
  #[inline]
  pub fn clear(&mut self) {
    self.len = 0;
  }

  /// Iterates over the slice `other`, copies each element, and then appends
  /// it to this vector. The `other` slice is traversed in-order.
  #[inline]
  pub fn extend_from_iter(
    &mut self,
    iter: impl IntoIterator<Item = D>,
  ) -> Result<(), ArrayVectorError> {
    for elem in iter {
      self.push(elem)?;
    }
    Ok(())
  }

  /// Return the inner fixed size array, if the capacity is full.
  #[inline]
  pub fn into_inner(self) -> Result<[D; N], ArrayVectorError> {
    if *Usize::from(self.len) >= N {
      // SAFETY: All elements are initialized
      Ok(unsafe { ptr::read(self.data.as_ptr().cast()) })
    } else {
      Err(ArrayVectorError::IntoInnerIncomplete)
    }
  }

  /// Shortens the vector, removing the last element.
  #[inline]
  pub fn pop(&mut self) -> bool {
    if let Some(elem) = self.len.checked_sub(1) {
      self.len = elem;
      true
    } else {
      false
    }
  }

  /// How many elements can be added to this collection.
  #[inline]
  pub fn remaining(&self) -> u32 {
    self.capacity().wrapping_sub(self.len)
  }

  /// Appends an element to the back of the collection.
  #[inline]
  pub fn push(&mut self, value: D) -> Result<(), ArrayVectorError> {
    let Some(elem) = self.data.get_mut(*Usize::from(self.len)) else {
      return Err(ArrayVectorError::PushOverflow);
    };
    *elem = MaybeUninit::new(value);
    self.len = self.len.wrapping_add(1);
    Ok(())
  }

  /// Shortens the vector, keeping the first `len` elements.
  #[inline]
  pub fn truncate(&mut self, len: u32) {
    self.len = len.min(self.capacity());
  }

  #[inline]
  const fn as_ptr(&self) -> *const D {
    self.data.as_ptr().cast()
  }

  #[inline]
  fn as_mut_ptr(&mut self) -> *mut D {
    self.data.as_mut_ptr().cast()
  }

  #[inline]
  const fn instance_check() {
    const {
      assert!(N <= Usize::from_u32(u32::MAX).into_usize() && !needs_drop::<D>());
    }
  }

  #[inline]
  const fn instance_u32() -> u32 {
    const {
      let [_, _, _, _, a, b, c, d] = Usize::from_usize(N).into_u64().to_be_bytes();
      u32::from_be_bytes([a, b, c, d])
    }
  }
}

impl<D, const N: usize> ArrayVector<D, N>
where
  D: Clone,
{
  /// Creates a new instance with the copyable elements of `slice`.
  #[inline]
  pub fn from_cloneable_slice(slice: &[D]) -> Result<Self, ArrayVectorError> {
    let mut this = Self::new();
    this.extend_from_cloneable_slice(slice)?;
    Ok(this)
  }

  /// Iterates over the slice `other`, copies each element, and then appends
  /// it to this vector. The `other` slice is traversed in-order.
  #[inline]
  pub fn extend_from_cloneable_slice(&mut self, other: &[D]) -> Result<(), ArrayVectorError> {
    for elem in other {
      self.push(elem.clone())?;
    }
    Ok(())
  }
}

impl<D, const N: usize> ArrayVector<D, N>
where
  D: Copy,
{
  /// Creates a new instance with the copyable elements of `slice`.
  #[inline]
  pub fn from_copyable_slice(slice: &[D]) -> Result<Self, ArrayVectorError> {
    let mut this = Self::new();
    this.extend_from_copyable_slice(slice)?;
    Ok(this)
  }

  /// Iterates over the slice `other`, copies each element, and then appends
  /// it to this vector. The `other` slice is traversed in-order.
  #[inline]
  pub fn extend_from_copyable_slice(&mut self, other: &[D]) -> Result<(), ArrayVectorError> {
    let this_len_u32 = self.len;
    let other_len = other.len();
    let Some(len_u32) = u32::try_from(other_len).ok().filter(|el| self.remaining() >= *el) else {
      return Err(ArrayVectorError::ExtendFromSliceOverflow);
    };
    // SAFETY: The above check ensures bounds
    let dst = unsafe { self.as_mut_ptr().add(*Usize::from_u32(this_len_u32)) };
    // SAFETY: Parameters are valid
    unsafe {
      ptr::copy_nonoverlapping(other.as_ptr(), dst, other_len);
    }
    self.len = self.len.wrapping_add(len_u32);
    Ok(())
  }
}

impl<D, const N: usize> Clone for ArrayVector<D, N>
where
  D: Clone,
{
  #[inline]
  fn clone(&self) -> Self {
    let mut this = Self::new();
    let _rslt = this.extend_from_cloneable_slice(self);
    this
  }
}

impl<D, const N: usize> Debug for ArrayVector<D, N>
where
  D: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    self.lease().fmt(f)
  }
}

impl<D, const N: usize> Default for ArrayVector<D, N> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<D, const N: usize> Deref for ArrayVector<D, N> {
  type Target = [D];

  #[inline]
  fn deref(&self) -> &Self::Target {
    // SAFETY: `len` ensures initialized elements
    unsafe { slice::from_raw_parts(self.as_ptr(), *Usize::from(self.len)) }
  }
}

impl<D, const N: usize> DerefMut for ArrayVector<D, N> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    // SAFETY: `len` ensures initialized elements
    unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), *Usize::from(self.len)) }
  }
}

impl<D, const N: usize> Eq for ArrayVector<D, N> where D: Eq {}

impl<D, const N: usize> IntoIterator for ArrayVector<D, N> {
  type IntoIter =
    iter::Map<iter::Take<array::IntoIter<MaybeUninit<D>, N>>, fn(MaybeUninit<D>) -> D>;
  type Item = D;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    fn map<D>(elem: MaybeUninit<D>) -> D {
      // SAFETY: Only maps initialized elements
      unsafe { elem.assume_init() }
    }
    self.data.into_iter().take(*Usize::from(self.len)).map(map)
  }
}

impl<'any, D, const N: usize> IntoIterator for &'any ArrayVector<D, N>
where
  D: 'any,
{
  type IntoIter = slice::Iter<'any, D>;
  type Item = &'any D;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

impl<'any, D, const N: usize> IntoIterator for &'any mut ArrayVector<D, N>
where
  D: 'any,
{
  type IntoIter = slice::IterMut<'any, D>;
  type Item = &'any mut D;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.iter_mut()
  }
}

impl<D, const N: usize> Lease<[D]> for ArrayVector<D, N> {
  #[inline]
  fn lease(&self) -> &[D] {
    self
  }
}

impl<D, const N: usize> LeaseMut<[D]> for ArrayVector<D, N> {
  #[inline]
  fn lease_mut(&mut self) -> &mut [D] {
    self
  }
}

impl<D, const N: usize> PartialEq for ArrayVector<D, N>
where
  D: PartialEq,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

impl<D, const N: usize> PartialEq<[D]> for ArrayVector<D, N>
where
  D: PartialEq,
{
  #[inline]
  fn eq(&self, other: &[D]) -> bool {
    **self == *other
  }
}

impl<D, const N: usize> PartialOrd for ArrayVector<D, N>
where
  D: PartialOrd,
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
    (**self).partial_cmp(&**other)
  }
}

impl<D, const N: usize> Ord for ArrayVector<D, N>
where
  D: Ord,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    (**self).cmp(other)
  }
}

impl<D, const N: usize> From<[D; N]> for ArrayVector<D, N> {
  #[inline]
  fn from(from: [D; N]) -> Self {
    Self::from_parts(from, Self::instance_u32())
  }
}

impl<const N: usize> fmt::Write for ArrayVector<u8, N> {
  #[inline]
  fn write_char(&mut self, c: char) -> fmt::Result {
    self.extend_from_copyable_slice(char_slice(&mut [0; 4], c)).map_err(|_err| fmt::Error)
  }

  #[inline]
  fn write_str(&mut self, s: &str) -> fmt::Result {
    self.extend_from_copyable_slice(s.as_bytes()).map_err(|_err| fmt::Error)
  }
}

#[cfg(feature = "std")]
impl<const N: usize> std::io::Write for ArrayVector<u8, N> {
  #[inline]
  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }

  #[inline]
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    let len = (*Usize::from(self.remaining())).min(buf.len());
    let _rslt = self.extend_from_copyable_slice(buf.get(..len).unwrap_or_default());
    Ok(len)
  }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
  use crate::misc::{ArrayVector, Usize};
  use arbitrary::{Arbitrary, Unstructured};

  impl<'any, T, const N: usize> Arbitrary<'any> for ArrayVector<T, N>
  where
    T: Arbitrary<'any>,
  {
    #[inline]
    fn arbitrary(u: &mut Unstructured<'any>) -> arbitrary::Result<Self> {
      let mut len = const {
        let [_, _, _, _, a, b, c, d] = Usize::from_usize(N).into_u64().to_be_bytes();
        u32::from_be_bytes([a, b, c, d])
      };
      len = u32::arbitrary(u)?.min(len);
      let mut this = Self::new();
      for _ in 0..len {
        let _rslt = this.push(T::arbitrary(u)?);
      }
      Ok(this)
    }
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::misc::ArrayVector;
  use core::{fmt::Formatter, marker::PhantomData};
  use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeTuple,
    Deserialize, Deserializer, Serialize, Serializer,
  };

  impl<'de, D, const N: usize> Deserialize<'de> for ArrayVector<D, N>
  where
    D: Deserialize<'de>,
  {
    #[inline]
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
      DE: Deserializer<'de>,
    {
      struct ArrayVisitor<D, const N: usize>(PhantomData<D>);

      impl<'de, D, const N: usize> Visitor<'de> for ArrayVisitor<D, N>
      where
        D: Deserialize<'de>,
      {
        type Value = ArrayVector<D, N>;

        #[inline]
        fn expecting(&self, formatter: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
          formatter.write_fmt(format_args!("an array with {N} elements"))
        }

        #[inline]
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
          A: SeqAccess<'de>,
        {
          let mut this = ArrayVector::new();
          for elem in &mut this {
            *elem = seq.next_element::<D>()?.ok_or_else(|| {
              de::Error::invalid_length(N, &"Array need more data to be constructed")
            })?;
          }
          Ok(this)
        }
      }

      deserializer.deserialize_tuple(N, ArrayVisitor::<D, N>(PhantomData))
    }
  }

  impl<D, const N: usize> Serialize for ArrayVector<D, N>
  where
    D: Serialize,
  {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      let mut seq = serializer.serialize_tuple(N)?;
      for elem in self.iter() {
        seq.serialize_element(elem)?;
      }
      seq.end()
    }
  }
}
