use alloc::string::String;
use core::hash::BuildHasher;

/// Statement command can be a string or the hashed contents of a string.
pub trait StmtCmd {
  /// Source command, if any.
  fn cmd(&self) -> Option<&str>;

  /// Based on the inner contents.
  fn hash(&self, hasher: &mut impl BuildHasher) -> u64;
}

impl<T> StmtCmd for &T
where
  T: StmtCmd,
{
  #[inline]
  fn cmd(&self) -> Option<&str> {
    (**self).cmd()
  }

  #[inline]
  fn hash(&self, hasher: &mut impl BuildHasher) -> u64 {
    (**self).hash(hasher)
  }
}

impl<T> StmtCmd for &mut T
where
  T: StmtCmd,
{
  #[inline]
  fn cmd(&self) -> Option<&str> {
    (**self).cmd()
  }

  #[inline]
  fn hash(&self, hasher: &mut impl BuildHasher) -> u64 {
    (**self).hash(hasher)
  }
}

impl StmtCmd for u64 {
  #[inline]
  fn cmd(&self) -> Option<&str> {
    None
  }

  #[inline]
  fn hash(&self, _: &mut impl BuildHasher) -> u64 {
    *self
  }
}

impl StmtCmd for &str {
  #[inline]
  fn cmd(&self) -> Option<&str> {
    Some(self)
  }

  #[inline]
  fn hash(&self, hasher: &mut impl BuildHasher) -> u64 {
    hasher.hash_one(self)
  }
}

impl StmtCmd for String {
  #[inline]
  fn cmd(&self) -> Option<&str> {
    Some(self)
  }

  #[inline]
  fn hash(&self, hasher: &mut impl BuildHasher) -> u64 {
    hasher.hash_one(self)
  }
}
