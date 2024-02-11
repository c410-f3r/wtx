/// An enum that can contain two different types.
#[derive(Debug, PartialEq)]
pub enum Either<L, R> {
  /// Left
  Left(L),
  /// Right
  Right(R),
}

impl<L, R> AsRef<[u8]> for Either<L, R>
where
  L: AsRef<[u8]>,
  R: AsRef<[u8]>,
{
  #[inline]
  fn as_ref(&self) -> &[u8] {
    match self {
      Either::Left(elem) => elem.as_ref(),
      Either::Right(elem) => elem.as_ref(),
    }
  }
}

impl<'any, L, R> AsRef<&'any [u8]> for Either<L, R>
where
  L: AsRef<&'any [u8]>,
  R: AsRef<&'any [u8]>,
{
  #[inline]
  fn as_ref(&self) -> &&'any [u8] {
    match self {
      Either::Left(elem) => elem.as_ref(),
      Either::Right(elem) => elem.as_ref(),
    }
  }
}

impl<L, R> AsRef<str> for Either<L, R>
where
  L: AsRef<str>,
  R: AsRef<str>,
{
  #[inline]
  fn as_ref(&self) -> &str {
    match self {
      Either::Left(elem) => elem.as_ref(),
      Either::Right(elem) => elem.as_ref(),
    }
  }
}

impl<'any, L, R> AsRef<&'any str> for Either<L, R>
where
  L: AsRef<&'any str>,
  R: AsRef<&'any str>,
{
  #[inline]
  fn as_ref(&self) -> &&'any str {
    match self {
      Either::Left(elem) => elem.as_ref(),
      Either::Right(elem) => elem.as_ref(),
    }
  }
}
