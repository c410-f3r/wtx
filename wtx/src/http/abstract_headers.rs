use crate::misc::{Block, BlocksQueue, _unreachable};
use core::ops::Range;

const DFLT_MAX_BYTES: usize = 4 * 1024;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct AbstractHeader<'ah, M> {
  pub(crate) is_sensitive: bool,
  pub(crate) misc: &'ah M,
  pub(crate) name_bytes: &'ah [u8],
  pub(crate) name_range: Range<usize>,
  pub(crate) value_bytes: &'ah [u8],
}

#[derive(Debug)]
pub(crate) struct AbstractHeaders<M> {
  max_bytes: usize,
  bq: BlocksQueue<u8, Metadata<M>>,
}

impl<M> AbstractHeaders<M>
where
  M: Copy,
{
  #[inline]
  pub(crate) const fn new(max_bytes: usize) -> Self {
    Self { max_bytes, bq: BlocksQueue::new() }
  }

  #[inline]
  pub(crate) fn with_capacity(bytes: usize, headers: usize, max_bytes: usize) -> Self {
    Self { max_bytes, bq: BlocksQueue::with_capacity(bytes.min(max_bytes), headers) }
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
  pub(crate) fn elements_len(&self) -> usize {
    self.bq.blocks_len()
  }

  #[inline]
  pub(crate) fn first(&self) -> Option<AbstractHeader<'_, M>> {
    self.bq.first().map(Self::map)
  }

  #[inline]
  pub(crate) fn get_by_idx(&self, idx: usize) -> Option<AbstractHeader<'_, M>> {
    self.bq.get(idx).map(Self::map)
  }

  #[inline]
  pub(crate) fn iter(&self) -> impl Iterator<Item = AbstractHeader<'_, M>> {
    self.bq.iter().map(Self::map)
  }

  #[inline]
  pub(crate) fn last(&self) -> Option<AbstractHeader<'_, M>> {
    self.bq.last().map(Self::map)
  }

  #[inline]
  pub(crate) fn max_bytes(&self) -> usize {
    self.max_bytes
  }

  #[inline]
  pub(crate) fn pop_back(&mut self) {
    let _ = self.bq.pop_back();
  }

  #[inline]
  pub(crate) fn pop_front(&mut self) {
    let _ = self.bq.pop_front();
  }

  #[inline]
  pub(crate) fn push_front(&mut self, misc: M, name: &[u8], value: &[u8], is_sensitive: bool) {
    let local_len = name.len().wrapping_add(value.len());
    if local_len > self.max_bytes {
      self.clear();
      return;
    }
    self.remove_until_max_bytes(local_len);
    self.bq.push_front_within_cap([name, value], |start| Metadata {
      is_active: true,
      is_sensitive,
      misc,
      sep_idx: start.wrapping_add(name.len()),
    });
  }

  #[inline]
  pub(crate) fn remove_by_idx(&mut self, idx: usize) -> Option<()> {
    let elem = self.bq.get_mut(idx)?;
    elem.misc.is_active = false;
    Some(())
  }

  #[inline]
  pub(crate) fn reserve(&mut self, bytes: usize, headers: usize) {
    self.bq.reserve(bytes.min(self.bytes_len()), headers);
  }

  #[inline]
  pub(crate) fn set_max_bytes(&mut self, max_bytes: usize) {
    self.max_bytes = max_bytes;
    self.remove_until_max_bytes(0);
  }

  fn map<'this>(elem: Block<&'this [u8], &'this Metadata<M>>) -> AbstractHeader<'this, M> {
    AbstractHeader {
      is_sensitive: elem.misc.is_sensitive,
      misc: &elem.misc.misc,
      name_bytes: if let Some(elem) = elem.data.get(..elem.misc.sep_idx) {
        elem
      } else {
        _unreachable()
      },
      name_range: elem.range.start..elem.misc.sep_idx,
      value_bytes: if let Some(elem) = elem.data.get(elem.misc.sep_idx..) {
        elem
      } else {
        _unreachable()
      },
    }
  }

  #[inline]
  fn remove_until_max_bytes(&mut self, additional: usize) {
    while self.bytes_len().wrapping_add(additional) > self.max_bytes {
      self.pop_back();
    }
  }
}

impl<M> Default for AbstractHeaders<M>
where
  M: Copy,
{
  #[inline]
  fn default() -> Self {
    Self::new(DFLT_MAX_BYTES)
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Metadata<M> {
  is_active: bool,
  is_sensitive: bool,
  misc: M,
  sep_idx: usize,
}
