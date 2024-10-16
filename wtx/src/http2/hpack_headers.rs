use crate::misc::{Block, BlocksQueue};

#[derive(Debug)]
pub(crate) struct HpackHeaders<M> {
  bq: BlocksQueue<u8, Metadata<M>>,
  max_bytes: usize,
}

impl<M> HpackHeaders<M>
where
  M: Copy,
{
  #[inline]
  pub(crate) const fn new(max_bytes: usize) -> Self {
    Self { bq: BlocksQueue::new(), max_bytes }
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
  pub(crate) fn get_by_idx(&self, idx: usize) -> Option<AbstractHeader<'_, M>> {
    self.bq.get(idx).as_ref().map(Self::map)
  }

  #[inline]
  pub(crate) fn max_bytes(&self) -> usize {
    self.max_bytes
  }

  #[inline]
  pub(crate) fn push_front<'bytes, I>(
    &mut self,
    misc: M,
    name: &'bytes [u8],
    values: I,
    is_sensitive: bool,
    cb: impl FnMut(M),
  ) -> crate::Result<()>
  where
    I: IntoIterator<Item = &'bytes [u8]>,
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
    self.bq.push_front(
      [name].into_iter().chain(iter),
      Metadata { is_sensitive, misc, name_len: name.len() },
    )?;
    Ok(())
  }

  #[inline(always)]
  pub(crate) fn reserve(&mut self, headers: usize, bytes: usize) -> crate::Result<()> {
    self.bq.reserve_front(headers, bytes)?;
    Ok(())
  }

  #[inline]
  pub(crate) fn set_max_bytes(&mut self, max_bytes: usize, cb: impl FnMut(M)) {
    self.max_bytes = max_bytes;
    self.remove_until_max_bytes(0, cb);
  }

  #[inline]
  fn map<'this>(block: &Block<&'this [u8], &'this Metadata<M>>) -> AbstractHeader<'this, M> {
    AbstractHeader {
      is_sensitive: block.misc.is_sensitive,
      misc: &block.misc.misc,
      name_bytes: block.data.get(..block.misc.name_len).unwrap_or_default(),
      value_bytes: block.data.get(block.misc.name_len..).unwrap_or_default(),
    }
  }

  #[inline]
  fn pop_back(&mut self) -> Option<Metadata<M>> {
    self.bq.pop_back()
  }

  #[inline]
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
  pub(crate) name_bytes: &'ah [u8],
  pub(crate) value_bytes: &'ah [u8],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Metadata<M> {
  is_sensitive: bool,
  name_len: usize,
  misc: M,
}
