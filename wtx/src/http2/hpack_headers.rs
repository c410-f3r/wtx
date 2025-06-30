use crate::collection::{Block, BlocksDeque};
use core::str;

#[derive(Debug)]
pub(crate) struct HpackHeaders<M> {
  bq: BlocksDeque<u8, Metadata<M>>,
  max_bytes: usize,
}

impl<M> HpackHeaders<M>
where
  M: Copy,
{
  pub(crate) const fn new(max_bytes: usize) -> Self {
    Self { bq: BlocksDeque::new(), max_bytes }
  }

  pub(crate) fn bytes_len(&self) -> usize {
    self.bq.elements_len()
  }

  pub(crate) fn clear(&mut self) {
    self.bq.clear();
  }

  pub(crate) fn headers_len(&self) -> usize {
    self.bq.blocks_len()
  }

  pub(crate) fn get_by_idx(&self, idx: usize) -> Option<AbstractHeader<'_, M>> {
    self.bq.get(idx).as_ref().map(Self::map)
  }

  pub(crate) fn max_bytes(&self) -> usize {
    self.max_bytes
  }

  pub(crate) fn push_front<'str, I>(
    &mut self,
    misc: M,
    name: &'str str,
    values: I,
    is_sensitive: bool,
    cb: impl FnMut(M),
  ) -> crate::Result<()>
  where
    I: IntoIterator<Item = &'str str>,
    I::IntoIter: Clone + ExactSizeIterator,
  {
    let iter = values.into_iter();
    let mut local_len = name.len();
    for elem in iter.clone() {
      local_len = local_len.wrapping_add(elem.len());
    }
    if local_len > self.max_bytes {
      self.clear();
      return Ok(());
    }
    self.remove_until_max_bytes(local_len, cb);
    self.bq.push_front_from_copyable_data(
      [name].into_iter().chain(iter).map(|el| el.as_bytes()),
      Metadata { is_sensitive, misc, name_len: name.len() },
    )?;
    Ok(())
  }

  #[inline(always)]
  pub(crate) fn reserve(&mut self, headers: usize, bytes: usize) -> crate::Result<()> {
    self.bq.reserve_front(headers, bytes)?;
    Ok(())
  }

  pub(crate) fn set_max_bytes(&mut self, max_bytes: usize, cb: impl FnMut(M)) {
    self.max_bytes = max_bytes;
    self.remove_until_max_bytes(0, cb);
  }

  fn map<'this>(block: &Block<&'this [u8], &'this Metadata<M>>) -> AbstractHeader<'this, M> {
    AbstractHeader {
      is_sensitive: block.misc.is_sensitive,
      misc: &block.misc.misc,
      name: {
        let str = block.data.get(..block.misc.name_len).unwrap_or_default();
        // SAFETY: input methods only accept UTF-8 data
        unsafe { str::from_utf8_unchecked(str) }
      },
      value: {
        let str = block.data.get(block.misc.name_len..).unwrap_or_default();
        // SAFETY: input methods only accept UTF-8 data
        unsafe { str::from_utf8_unchecked(str) }
      },
    }
  }

  fn pop_back(&mut self) -> Option<Metadata<M>> {
    self.bq.pop_back()
  }

  fn remove_until_max_bytes(&mut self, additional: usize, mut cb: impl FnMut(M)) {
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
  name_len: usize,
  misc: M,
}
