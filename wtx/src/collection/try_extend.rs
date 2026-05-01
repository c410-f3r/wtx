use crate::{
  collection::{ArrayString, ArrayVector, ExpansionTy, LinearStorageLen, Vector},
  misc::{Wrapper, from_utf8_basic},
};
use alloc::{string::String, vec::Vec};

/// Extends collections in a fallible way, i.e., checks if the upcoming elements would exceed
/// the associated capacity.
pub trait TryExtend<S> {
  /// If the implementation is of type `()`. In other words, a dummy type.
  const IS_UNIT: bool = false;

  /// Attempts to extend this instance with elements from the given `set` source.
  fn try_extend(&mut self, set: S) -> crate::Result<()>;
}

impl<S, T> TryExtend<S> for &mut T
where
  T: TryExtend<S>,
{
  const IS_UNIT: bool = T::IS_UNIT;

  #[inline]
  fn try_extend(&mut self, set: S) -> crate::Result<()> {
    (**self).try_extend(set)
  }
}

// ArrayString

impl<L, const N: usize> TryExtend<(u8, usize)> for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn try_extend(&mut self, (elem, len): (u8, usize)) -> crate::Result<()> {
    self.extend_from_iter((0..len).map(|_| char::from(elem)))?;
    Ok(())
  }
}

impl<'slice, L, const N: usize> TryExtend<&'slice str> for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn try_extend(&mut self, set: &'slice str) -> crate::Result<()> {
    self.push_str(set)?;
    Ok(())
  }
}

impl<'slice, L, const N: usize> TryExtend<&'slice [u8]> for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn try_extend(&mut self, set: &'slice [u8]) -> crate::Result<()> {
    self.push_str(from_utf8_basic(set)?)?;
    Ok(())
  }
}

impl<L, const M: usize, const N: usize> TryExtend<[u8; M]> for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn try_extend(&mut self, set: [u8; M]) -> crate::Result<()> {
    self.push_str(from_utf8_basic(&set)?)?;
    Ok(())
  }
}

impl<I, L, const N: usize> TryExtend<Wrapper<I>> for ArrayString<L, N>
where
  I: IntoIterator<Item = char>,
  L: LinearStorageLen,
{
  #[inline]
  fn try_extend(&mut self, set: Wrapper<I>) -> crate::Result<()> {
    self.extend_from_iter(set.0)?;
    Ok(())
  }
}

// ArrayVector

impl<L, T, const N: usize> TryExtend<(T, usize)> for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
  T: Clone,
{
  #[inline]
  fn try_extend(&mut self, (elem, len): (T, usize)) -> crate::Result<()> {
    self.expand(ExpansionTy::Additional(len), elem)?;
    Ok(())
  }
}

impl<'slice, L, T, const N: usize> TryExtend<&'slice [T]> for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
  T: Copy,
{
  #[inline]
  fn try_extend(&mut self, set: &'slice [T]) -> crate::Result<()> {
    self.extend_from_copyable_slice(set)?;
    Ok(())
  }
}

impl<L, T, const M: usize, const N: usize> TryExtend<[T; M]> for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn try_extend(&mut self, set: [T; M]) -> crate::Result<()> {
    self.extend_from_iter(set)?;
    Ok(())
  }
}

impl<I, L, T, const N: usize> TryExtend<Wrapper<I>> for ArrayVector<L, T, N>
where
  I: IntoIterator<Item = T>,
  L: LinearStorageLen,
{
  #[inline]
  fn try_extend(&mut self, set: Wrapper<I>) -> crate::Result<()> {
    self.extend_from_iter(set.0)?;
    Ok(())
  }
}

// Option

impl<'slice, T> TryExtend<&'slice [T]> for Option<T>
where
  T: Copy,
{
  #[inline]
  fn try_extend(&mut self, set: &'slice [T]) -> crate::Result<()> {
    if set.is_empty() {
      return Ok(());
    }
    let (None, [elem]) = (&*self, set) else {
      return Err(crate::Error::InsufficientOptionCapacity);
    };
    *self = Some(*elem);
    Ok(())
  }
}

impl<T, const M: usize> TryExtend<[T; M]> for Option<T> {
  #[inline]
  fn try_extend(&mut self, set: [T; M]) -> crate::Result<()> {
    if set.is_empty() {
      return Ok(());
    }
    let mut iter = set.into_iter();
    let (None, Some(elem), None) = (&*self, iter.next(), iter.next()) else {
      return Err(crate::Error::InsufficientOptionCapacity);
    };
    *self = Some(elem);
    Ok(())
  }
}

impl<I, T> TryExtend<Wrapper<I>> for Option<T>
where
  I: IntoIterator<Item = T>,
{
  #[inline]
  fn try_extend(&mut self, set: Wrapper<I>) -> crate::Result<()> {
    let mut iter = set.0.into_iter();
    let Some(elem) = iter.next() else {
      return Ok(());
    };
    let (None, None) = (&*self, iter.next()) else {
      return Err(crate::Error::InsufficientOptionCapacity);
    };
    *self = Some(elem);
    Ok(())
  }
}

// String

impl TryExtend<(u8, usize)> for String {
  #[inline]
  fn try_extend(&mut self, (elem, len): (u8, usize)) -> crate::Result<()> {
    self.extend((0..len).map(|_| char::from(elem)));
    Ok(())
  }
}

impl<'slice> TryExtend<&'slice str> for String {
  #[inline]
  fn try_extend(&mut self, set: &'slice str) -> crate::Result<()> {
    self.push_str(set);
    Ok(())
  }
}

impl<'slice> TryExtend<&'slice [u8]> for String {
  #[inline]
  fn try_extend(&mut self, set: &'slice [u8]) -> crate::Result<()> {
    self.push_str(from_utf8_basic(set)?);
    Ok(())
  }
}

impl<const M: usize> TryExtend<[u8; M]> for String {
  #[inline]
  fn try_extend(&mut self, set: [u8; M]) -> crate::Result<()> {
    self.push_str(from_utf8_basic(&set)?);
    Ok(())
  }
}

impl<I> TryExtend<Wrapper<I>> for String
where
  I: IntoIterator<Item = char>,
{
  #[inline]
  fn try_extend(&mut self, set: Wrapper<I>) -> crate::Result<()> {
    self.extend(set.0);
    Ok(())
  }
}

// Unit

impl<'slice, T> TryExtend<&'slice [T]> for ()
where
  T: Copy,
{
  const IS_UNIT: bool = true;

  #[inline]
  fn try_extend(&mut self, _: &'slice [T]) -> crate::Result<()> {
    Ok(())
  }
}

impl<T, const M: usize> TryExtend<[T; M]> for () {
  const IS_UNIT: bool = true;

  #[inline]
  fn try_extend(&mut self, _: [T; M]) -> crate::Result<()> {
    Ok(())
  }
}

impl<I, T> TryExtend<Wrapper<I>> for ()
where
  I: IntoIterator<Item = T>,
{
  const IS_UNIT: bool = true;

  #[inline]
  fn try_extend(&mut self, _: Wrapper<I>) -> crate::Result<()> {
    Ok(())
  }
}

// Vec

impl<T> TryExtend<(T, usize)> for Vec<T>
where
  T: Clone,
{
  #[inline]
  fn try_extend(&mut self, (elem, len): (T, usize)) -> crate::Result<()> {
    self.resize(len, elem);
    Ok(())
  }
}

impl<'slice, T> TryExtend<&'slice [T]> for Vec<T>
where
  T: Copy,
{
  #[inline]
  fn try_extend(&mut self, set: &'slice [T]) -> crate::Result<()> {
    self.copy_from_slice(set);
    Ok(())
  }
}

impl<T, const N: usize> TryExtend<[T; N]> for Vec<T> {
  #[inline]
  fn try_extend(&mut self, set: [T; N]) -> crate::Result<()> {
    self.extend(set);
    Ok(())
  }
}

impl<I, T> TryExtend<Wrapper<I>> for Vec<T>
where
  I: IntoIterator<Item = T>,
{
  #[inline]
  fn try_extend(&mut self, set: Wrapper<I>) -> crate::Result<()> {
    self.extend(set.0);
    Ok(())
  }
}

// Vector

impl<T> TryExtend<(T, usize)> for Vector<T>
where
  T: Clone,
{
  #[inline]
  fn try_extend(&mut self, (elem, len): (T, usize)) -> crate::Result<()> {
    self.expand(ExpansionTy::Additional(len), elem)?;
    Ok(())
  }
}

impl<'slice, T> TryExtend<&'slice [T]> for Vector<T>
where
  T: Copy,
{
  #[inline]
  fn try_extend(&mut self, set: &'slice [T]) -> crate::Result<()> {
    self.extend_from_copyable_slice(set)?;
    Ok(())
  }
}

impl<T, const N: usize> TryExtend<[T; N]> for Vector<T> {
  #[inline]
  fn try_extend(&mut self, set: [T; N]) -> crate::Result<()> {
    self.extend_from_iter(set)?;
    Ok(())
  }
}

impl<I, T> TryExtend<Wrapper<I>> for Vector<T>
where
  I: IntoIterator<Item = T>,
{
  #[inline]
  fn try_extend(&mut self, set: Wrapper<I>) -> crate::Result<()> {
    self.extend_from_iter(set.0)?;
    Ok(())
  }
}
