use crate::misc::Lease;

/// A `Cow` version without `alloc` and `ToOwned`.
pub type RefOrOwned<'any, T> = Either<&'any T, T>;

/// An enum that can contain two different types and only one is selected at runtime, in other
/// words, a choice.
///
/// <i>Choice, the problem is choice.</i>
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Either<L, R> {
  /// Left
  Left(L),
  /// Right
  Right(R),
}

impl<L, R> Lease<[u8]> for Either<L, R>
where
  L: Lease<[u8]>,
  R: Lease<[u8]>,
{
  #[inline]
  fn lease(&self) -> &[u8] {
    match self {
      Either::Left(elem) => elem.lease(),
      Either::Right(elem) => elem.lease(),
    }
  }
}

impl<T> Lease<T> for Either<&T, T> {
  #[inline]
  fn lease(&self) -> &T {
    match self {
      Either::Left(elem) => elem,
      Either::Right(elem) => elem,
    }
  }
}

impl<'any, L, R> Lease<&'any [u8]> for Either<L, R>
where
  L: Lease<&'any [u8]>,
  R: Lease<&'any [u8]>,
{
  #[inline]
  fn lease(&self) -> &&'any [u8] {
    match self {
      Either::Left(elem) => elem.lease(),
      Either::Right(elem) => elem.lease(),
    }
  }
}

impl<L, R> Lease<str> for Either<L, R>
where
  L: Lease<str>,
  R: Lease<str>,
{
  #[inline]
  fn lease(&self) -> &str {
    match self {
      Either::Left(elem) => elem.lease(),
      Either::Right(elem) => elem.lease(),
    }
  }
}

impl<'any, L, R> Lease<&'any str> for Either<L, R>
where
  L: Lease<&'any str>,
  R: Lease<&'any str>,
{
  #[inline]
  fn lease(&self) -> &&'any str {
    match self {
      Either::Left(elem) => elem.lease(),
      Either::Right(elem) => elem.lease(),
    }
  }
}
