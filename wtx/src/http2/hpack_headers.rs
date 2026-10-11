use crate::{
  collections::{Block, BlocksDequeU32},
  misc::Usize,
};
use core::str;

#[derive(Debug, Default)]
pub(crate) struct HpackHeaders<M> {
  bd: BlocksDequeU32<u8, Metadata<M>>,
  max_bytes: u32,
}

impl<M> HpackHeaders<M>
where
  M: Copy,
{
  pub(crate) const fn new(max_bytes: u32) -> Self {
    Self { bd: BlocksDequeU32::new(), max_bytes }
  }

  pub(crate) fn bytes_len(&self) -> u32 {
    self.bd.elements_len().wrapping_add(self.bd.blocks_len().wrapping_mul(32))
  }

  pub(crate) fn clear(&mut self) {
    self.bd.clear();
  }

  pub(crate) fn headers_len(&self) -> u32 {
    self.bd.blocks_len()
  }

  pub(crate) fn get_by_idx(&self, idx: u32) -> Option<AbstractHeader<'_, M>> {
    self.bd.get(idx).as_ref().map(Self::map)
  }

  pub(crate) const fn max_bytes(&self) -> u32 {
    self.max_bytes
  }

  pub(crate) fn push_front<'str, I>(
    &mut self,
    is_sensitive: bool,
    misc: M,
    name: &'str str,
    values: I,
    cb: impl FnMut(M),
  ) -> crate::Result<()>
  where
    I: IntoIterator<Item = &'str str>,
    I::IntoIter: Clone + ExactSizeIterator,
  {
    let iter = values.into_iter();
    let name_len: u32 = name.len().try_into()?;
    let mut total_len = name_len.wrapping_add(32);
    for elem in iter.clone() {
      total_len = total_len.wrapping_add(elem.len().try_into()?);
    }
    if total_len > self.max_bytes {
      self.clear();
      return Ok(());
    }
    self.remove_until_max_bytes(total_len, cb);
    self.bd.push_front_from_copyable_data(
      [name].into_iter().chain(iter).map(str::as_bytes),
      Metadata { is_sensitive, misc, name_len },
    )?;
    Ok(())
  }

  pub(crate) fn set_max_bytes(&mut self, max_bytes: u32, cb: impl FnMut(M)) {
    self.max_bytes = max_bytes;
    self.remove_until_max_bytes(0, cb);
  }

  fn map<'this>(block: &Block<&'this [u8], u32, &'this Metadata<M>>) -> AbstractHeader<'this, M> {
    AbstractHeader {
      is_sensitive: block.misc.is_sensitive,
      misc: &block.misc.misc,
      name: {
        let str = block.data.get(..*Usize::from(block.misc.name_len)).unwrap_or_default();
        // SAFETY: input methods only accept UTF-8 data
        unsafe { str::from_utf8_unchecked(str) }
      },
      value: {
        let str = block.data.get(*Usize::from(block.misc.name_len)..).unwrap_or_default();
        // SAFETY: input methods only accept UTF-8 data
        unsafe { str::from_utf8_unchecked(str) }
      },
    }
  }

  fn pop_back(&mut self) -> Option<Metadata<M>> {
    self.bd.pop_back()
  }

  fn remove_until_max_bytes(&mut self, additional: u32, mut cb: impl FnMut(M)) {
    while self.bytes_len().wrapping_add(additional) > self.max_bytes {
      if let Some(elem) = self.pop_back() {
        cb(elem.misc);
      }
    }
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AbstractHeader<'ah, M> {
  pub(crate) is_sensitive: bool,
  pub(crate) misc: &'ah M,
  pub(crate) name: &'ah str,
  pub(crate) value: &'ah str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Metadata<M> {
  is_sensitive: bool,
  misc: M,
  name_len: u32,
}
