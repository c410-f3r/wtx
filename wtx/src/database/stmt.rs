use core::hash::BuildHasher;

/// Statement identifier. Can be a string or the hashed contents of a string.
pub trait StmtId {
  /// Source command, if any.
  fn cmd(&self) -> Option<&str>;

  /// Based on the inner contents.
  fn hash(&self, hasher: &mut impl BuildHasher) -> u64;
}

impl<T> StmtId for &T
where
  T: StmtId,
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

impl StmtId for u64 {
  #[inline]
  fn cmd(&self) -> Option<&str> {
    None
  }

  #[inline]
  fn hash(&self, _: &mut impl BuildHasher) -> u64 {
    *self
  }
}

impl StmtId for &str {
  #[inline]
  fn cmd(&self) -> Option<&str> {
    Some(self)
  }

  #[inline]
  fn hash(&self, hasher: &mut impl BuildHasher) -> u64 {
    hasher.hash_one(self)
  }
}
