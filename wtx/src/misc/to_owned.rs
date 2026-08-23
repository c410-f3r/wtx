use crate::{
  collections::{LinearStorageLen, ShortBoxStr},
  misc::Lease,
};
use alloc::string::String;
use core::mem;

/// A generalization of Clone to borrowed data.
pub trait ToOwned<O>
where
  O: Lease<Self>,
{
  /// Creates owned data from borrowed data, usually by cloning.
  fn to_owned(&self) -> crate::Result<O>;

  /// Uses borrowed data to replace owned data, usually by cloning.
  #[inline]
  fn clone_into(&self, target: &mut O) -> crate::Result<()> {
    *target = self.to_owned()?;
    Ok(())
  }
}

impl<L> ToOwned<ShortBoxStr<L>> for str
where
  L: LinearStorageLen,
{
  #[inline]
  fn to_owned(&self) -> crate::Result<ShortBoxStr<L>> {
    ShortBoxStr::try_from(self)
  }

  #[inline]
  fn clone_into(&self, target: &mut ShortBoxStr<L>) -> crate::Result<()> {
    let mut string: String = mem::take(target).into();
    string.clear();
    string.push_str(self);
    *target = string.try_into()?;
    Ok(())
  }
}

impl ToOwned<String> for str {
  #[inline]
  fn to_owned(&self) -> crate::Result<String> {
    Ok(self.into())
  }

  #[inline]
  fn clone_into(&self, target: &mut String) -> crate::Result<()> {
    target.clear();
    target.push_str(self);
    Ok(())
  }
}
