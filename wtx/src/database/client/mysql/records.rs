use crate::database::client::mysql::{Mysql, Record};
use core::marker::PhantomData;

/// Records
#[derive(Debug)]
pub struct Records<'exec, E> {
  pub(crate) bytes: &'exec [u8],
  pub(crate) phantom: PhantomData<fn() -> E>,
}

impl<'exec, E> crate::database::Records<'exec> for Records<'exec, E>
where
  E: From<crate::Error>,
{
  type Database = Mysql<E>;

  #[inline]
  fn get(&self, _: usize) -> Option<Record<'exec, E>> {
    None
  }

  #[inline]
  fn iter(&self) -> impl Iterator<Item = Record<'exec, E>> {
    [].into_iter()
  }

  #[inline]
  fn len(&self) -> usize {
    0
  }
}

impl<E> Default for Records<'_, E> {
  #[inline]
  fn default() -> Self {
    Self { bytes: &[], phantom: PhantomData }
  }
}
