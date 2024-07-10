use crate::misc::{Block, BlocksQueue, _unreachable};
use core::fmt::{Debug, Formatter};

pub(crate) struct AbstractHeaders<M> {
  bq: BlocksQueue<u8, Metadata<M>>,
  max_bytes: usize,
}

impl<M> AbstractHeaders<M>
where
  M: Copy,
{
  #[inline]
  pub(crate) const fn new(max_bytes: usize) -> Self {
    Self { bq: BlocksQueue::new(), max_bytes }
  }

  #[inline]
  pub(crate) fn with_capacity(
    bytes: usize,
    headers: usize,
    max_bytes: usize,
  ) -> crate::Result<Self> {
    Ok(Self { bq: BlocksQueue::with_capacity(headers, bytes.min(max_bytes))?, max_bytes })
  }

  #[inline]
  pub(crate) fn bytes_len(&self) -> usize {
    self.bq.elements_len()
  }

  #[inline]
  pub(crate) fn clear(&mut self) {
    self.bq.clear();
  }

  #[inline]
  pub(crate) fn headers_len(&self) -> usize {
    self.bq.blocks_len()
  }

  #[inline]
  pub(crate) fn first(&self) -> Option<AbstractHeader<'_, M>> {
    self.bq.first().as_ref().map(Self::map)
  }

  #[inline]
  pub(crate) fn get_by_idx(&self, idx: usize) -> Option<AbstractHeader<'_, M>> {
    self.bq.get(idx).as_ref().map(Self::map)
  }

  #[inline]
  pub(crate) fn get_by_name(&self, name: &[u8]) -> Option<AbstractHeader<'_, M>> {
    self.iter().find(|el| el.name_bytes == name)
  }

  #[inline]
  pub(crate) fn iter(&self) -> impl Iterator<Item = AbstractHeader<'_, M>> {
    self.bq.iter().map(|el| Self::map(&el))
  }

  #[inline]
  pub(crate) fn last(&self) -> Option<AbstractHeader<'_, M>> {
    self.bq.last().as_ref().map(Self::map)
  }

  #[inline]
  pub(crate) fn max_bytes(&self) -> usize {
    self.max_bytes
  }

  #[inline]
  pub(crate) fn pop_back(&mut self) -> Option<(Metadata<M>, &mut [u8])> {
    self.bq.pop_back()
  }

  #[inline]
  pub(crate) fn pop_front(&mut self) -> Option<(Metadata<M>, &mut [u8])> {
    self.bq.pop_front()
  }

  #[inline]
  pub(crate) fn push_front(
    &mut self,
    misc: M,
    name: &[u8],
    [value0, value1]: [&[u8]; 2],
    is_sensitive: bool,
    cb: impl FnMut(M, &mut [u8]),
  ) -> crate::Result<()> {
    let local_len = name.len().wrapping_add(value0.len()).wrapping_add(value1.len());
    if local_len > self.max_bytes {
      self.clear();
      return Ok(());
    }
    self.remove_until_max_bytes(local_len, cb);
    self.bq.push_front(
      [name, value0, value1],
      Metadata { is_active: true, is_sensitive, misc, name_len: name.len() },
    )?;
    Ok(())
  }

  #[inline]
  pub(crate) fn remove_by_idx(&mut self, idx: usize) -> Option<()> {
    let elem = self.bq.get_mut(idx)?;
    elem.misc.is_active = false;
    Some(())
  }

  #[inline(always)]
  pub(crate) fn reserve(&mut self, bytes: usize, headers: usize) -> crate::Result<()> {
    self.bq.reserve(headers, bytes.min(self.max_bytes))?;
    Ok(())
  }

  #[inline]
  pub(crate) fn set_max_bytes(&mut self, max_bytes: usize, cb: impl FnMut(M, &mut [u8])) {
    self.max_bytes = max_bytes;
    self.remove_until_max_bytes(0, cb);
  }

  #[inline]
  fn map<'this>(block: &Block<&'this [u8], &'this Metadata<M>>) -> AbstractHeader<'this, M> {
    AbstractHeader {
      is_sensitive: block.misc.is_sensitive,
      misc: &block.misc.misc,
      name_bytes: if let Some(elem) = block.data.get(..block.misc.name_len) {
        elem
      } else {
        _unreachable()
      },
      value_bytes: if let Some(elem) = block.data.get(block.misc.name_len..) {
        elem
      } else {
        _unreachable()
      },
    }
  }

  #[inline]
  fn remove_until_max_bytes(&mut self, additional: usize, mut cb: impl FnMut(M, &mut [u8])) {
    while self.bytes_len().wrapping_add(additional) > self.max_bytes {
      if let Some(elem) = self.pop_back() {
        cb(elem.0.misc, elem.1);
      }
    }
  }
}

impl<M> Debug for AbstractHeaders<M>
where
  M: Copy + Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    f.debug_struct("AbstractHeaders")
      .field("max_bytes", &self.max_bytes)
      .field("bq", &self.bq)
      .finish()
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AbstractHeader<'ah, M> {
  pub(crate) is_sensitive: bool,
  pub(crate) misc: &'ah M,
  pub(crate) name_bytes: &'ah [u8],
  pub(crate) value_bytes: &'ah [u8],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Metadata<M> {
  is_active: bool,
  is_sensitive: bool,
  misc: M,
  name_len: usize,
}
