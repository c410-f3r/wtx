#![allow(
  // Indices point to valid memory
  clippy::unreachable
)]

use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

/// Internal buffer not intended for public usage
#[derive(Debug, Default)]
pub struct Buffer {
  buffer: Vec<u8>,
  len: usize,
}

impl Buffer {
  pub(crate) fn clear(&mut self) {
    self.len = 0;
  }

  pub(crate) fn push_bytes(&mut self, bytes: &[u8]) {
    let prev = self.len;
    let curr = prev.wrapping_add(bytes.len());
    self.set_idx_through_expansion(curr);
    self.get_mut(prev..curr).unwrap_or_default().copy_from_slice(bytes);
  }

  pub(crate) fn set_idx_through_expansion(&mut self, len: usize) {
    self.len = len;
    self.expand(len);
  }

  fn expand(&mut self, new_len: usize) {
    if new_len > self.buffer.len() {
      self.buffer.resize(new_len, 0);
    }
  }
}

impl Deref for Buffer {
  type Target = [u8];

  #[inline]
  fn deref(&self) -> &Self::Target {
    if let Some(el) = self.buffer.get(..self.len) {
      el
    } else {
      unreachable!()
    }
  }
}

impl DerefMut for Buffer {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    if let Some(el) = self.buffer.get_mut(..self.len) {
      el
    } else {
      unreachable!()
    }
  }
}
