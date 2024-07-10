use crate::misc::{Vector, _unreachable};
use core::ops::{Deref, DerefMut};

#[derive(Debug)]
pub(crate) struct FilledBuffer {
  buffer: Vector<u8>,
  len: usize,
}

impl FilledBuffer {
  pub(crate) fn with_capacity(capacity: usize) -> Self {
    Self { buffer: _vector![0; capacity], len: 0 }
  }

  pub(crate) fn clear(&mut self) {
    self.len = 0;
  }

  pub(crate) fn push_bytes(&mut self, bytes: &[u8]) -> crate::Result<()> {
    let prev = self.len;
    let curr = prev.wrapping_add(bytes.len());
    self.set_idx_through_expansion(curr)?;
    self.get_mut(prev..curr).unwrap_or_default().copy_from_slice(bytes);
    Ok(())
  }

  pub(crate) fn set_idx_through_expansion(&mut self, len: usize) -> crate::Result<()> {
    self.len = len;
    self.expand(len)
  }

  fn expand(&mut self, new_len: usize) -> crate::Result<()> {
    if new_len > self.buffer.len() {
      self.buffer.expand(new_len, 0)?;
    }
    Ok(())
  }
}

impl Default for FilledBuffer {
  #[inline]
  fn default() -> Self {
    Self::with_capacity(1024)
  }
}

impl Deref for FilledBuffer {
  type Target = [u8];

  #[inline]
  fn deref(&self) -> &Self::Target {
    if let Some(el) = self.buffer.get(..self.len) {
      el
    } else {
      _unreachable()
    }
  }
}

impl DerefMut for FilledBuffer {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    if let Some(el) = self.buffer.get_mut(..self.len) {
      el
    } else {
      _unreachable()
    }
  }
}
