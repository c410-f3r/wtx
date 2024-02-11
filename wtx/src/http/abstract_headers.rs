use crate::misc::{Usize, _unlikely_cb, _unlikely_elem};
use alloc::collections::VecDeque;
use core::ops::Range;

const DFLT_MAX_BYTES: u32 = 4 * 1024;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct AbstractHeader<'ah, M> {
  pub(crate) misc: &'ah M,
  pub(crate) name_bytes: &'ah [u8],
  pub(crate) name_range: Range<u32>,
  pub(crate) value_bytes: &'ah [u8],
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct AbstractHeaders<M> {
  buffer: VecDeque<u8>,
  elements_len: u32,
  first_idx: u32,
  max_bytes: u32,
  metadata: VecDeque<Metadata<M>>,
}

impl<M> AbstractHeaders<M> {
  pub(crate) fn with_capacity(len: u32) -> Self {
    Self {
      buffer: VecDeque::with_capacity(*Usize::from(len)),
      elements_len: 0,
      first_idx: 0,
      max_bytes: DFLT_MAX_BYTES,
      metadata: VecDeque::with_capacity(*Usize::from(len)),
    }
  }

  // Insertions are limited to u32
  #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
  pub(crate) fn bytes_len(&self) -> u32 {
    self.buffer.len() as u32
  }

  pub(crate) fn clear(&mut self) {
    let Self { buffer, elements_len, first_idx, max_bytes: _, metadata } = self;
    buffer.clear();
    *elements_len = 0;
    *first_idx = 0;
    metadata.clear();
  }

  pub(crate) fn elements_len(&self) -> u32 {
    self.elements_len
  }

  pub(crate) fn get_by_idx(&self, idx: usize) -> Option<AbstractHeader<'_, M>> {
    let Some(Metadata { is_activated, name_begin_idx, misc, sep_idx, value_end_idx }) =
      self.metadata.get(idx)
    else {
      return _unlikely_elem(None);
    };
    if !is_activated {
      return _unlikely_elem(None);
    }
    let Some(name_bytes) = self.buffer.as_slices().0.get(
      *Usize::from(name_begin_idx.wrapping_sub(self.first_idx))
        ..*Usize::from(sep_idx.wrapping_sub(self.first_idx)),
    ) else {
      return _unlikely_elem(None);
    };
    let Some(value_bytes) = self.buffer.as_slices().0.get(
      *Usize::from(sep_idx.wrapping_sub(self.first_idx))
        ..*Usize::from(value_end_idx.wrapping_sub(self.first_idx)),
    ) else {
      return _unlikely_elem(None);
    };
    Some(AbstractHeader { misc, name_bytes, name_range: *name_begin_idx..*sep_idx, value_bytes })
  }

  pub(crate) fn get_by_name(&self, name: &[u8]) -> Option<AbstractHeader<'_, M>> {
    self.iter().find(|elem| (name == elem.name_bytes))
  }

  pub(crate) fn iter(&self) -> impl Iterator<Item = AbstractHeader<'_, M>> {
    self.metadata.iter().filter_map(
      |Metadata { is_activated, name_begin_idx, misc, sep_idx, value_end_idx }| {
        if !is_activated {
          return None;
        }
        let Some(name_bytes) = self.buffer.as_slices().0.get(
          *Usize::from(name_begin_idx.wrapping_sub(self.first_idx))
            ..*Usize::from(sep_idx.wrapping_sub(self.first_idx)),
        ) else {
          return _unlikely_elem(None);
        };
        let Some(value_bytes) = self.buffer.as_slices().0.get(
          *Usize::from(sep_idx.wrapping_sub(self.first_idx))
            ..*Usize::from(value_end_idx.wrapping_sub(self.first_idx)),
        ) else {
          return _unlikely_elem(None);
        };
        Some(AbstractHeader {
          misc,
          name_bytes,
          name_range: *name_begin_idx..*sep_idx,
          value_bytes,
        })
      },
    )
  }

  pub(crate) fn max_bytes(&self) -> u32 {
    self.max_bytes
  }

  pub(crate) fn normalize_indcs(&mut self) {
    let mut iter = self.metadata.as_mut_slices().0.iter_mut();
    let first = if let Some(elem) = iter.next() {
      let first = elem.name_begin_idx;
      elem.name_begin_idx = elem.name_begin_idx.wrapping_sub(first);
      elem.sep_idx = elem.sep_idx.wrapping_sub(first);
      elem.value_end_idx = elem.value_end_idx.wrapping_sub(first);
      first
    } else {
      return;
    };
    for elem in iter {
      elem.name_begin_idx = elem.name_begin_idx.wrapping_sub(first);
      elem.sep_idx = elem.sep_idx.wrapping_sub(first);
      elem.value_end_idx = elem.value_end_idx.wrapping_sub(first);
    }
  }

  pub(crate) fn pop_back(&mut self) {
    let Some(Metadata { name_begin_idx, .. }) = self.metadata.pop_back() else {
      return;
    };
    self.buffer.truncate(*Usize::from(name_begin_idx));
    self.elements_len = self.elements_len.wrapping_sub(1);
  }

  pub(crate) fn pop_front(&mut self) {
    let Some(Metadata { value_end_idx, .. }) = self.metadata.pop_front() else {
      return;
    };
    for _ in 0..value_end_idx.wrapping_sub(self.first_idx) {
      let _ = self.buffer.pop_front();
    }
    self.elements_len = self.elements_len.wrapping_sub(1);
    self.first_idx = value_end_idx;
  }

  pub(crate) fn push(&mut self, misc: M, name: &[u8], value: &[u8]) {
    let local_len = name.len().wrapping_add(value.len());
    if local_len > *Usize::from(self.max_bytes) {
      self.clear();
      return;
    }
    while Usize::from(self.bytes_len()).wrapping_add(local_len) > *Usize::from(self.max_bytes) {
      self.pop_front();
    }
    if Usize::from(self.first_idx).overflowing_add(local_len).1 {
      _unlikely_cb(|| self.normalize_indcs());
    }
    let name_begin_idx = self.bytes_len();
    self.buffer.extend(name);
    let sep_idx = self.bytes_len();
    self.buffer.extend(value);
    let value_begin_idx = self.bytes_len();
    self.push_metadata(misc, name_begin_idx, sep_idx, value_begin_idx);
  }

  pub(crate) fn push_metadata(
    &mut self,
    misc: M,
    name_begin_idx: u32,
    sep_idx: u32,
    value_end_idx: u32,
  ) {
    self.elements_len = self.elements_len.wrapping_add(1);
    self.metadata.push_back(Metadata {
      is_activated: true,
      misc,
      name_begin_idx,
      sep_idx,
      value_end_idx,
    });
  }

  pub(crate) fn remove(&mut self, names: &[&[u8]]) {
    if names.is_empty() {
      return;
    }
    let mut names_start = 0;
    for metadata in &mut self.metadata {
      let Metadata { is_activated, name_begin_idx, misc: _, sep_idx, value_end_idx: _ } = metadata;
      if !*is_activated {
        continue;
      }
      let tuple = (
        self.buffer.as_slices().0.get(*Usize::from(*name_begin_idx)..*Usize::from(*sep_idx)),
        names.get(names_start..),
      );
      let (Some(name_bytes), Some(slice)) = tuple else {
        break;
      };
      if slice.contains(&name_bytes) {
        *is_activated = false;
        names_start = names_start.wrapping_add(1);
        self.elements_len = self.elements_len.wrapping_sub(1);
      }
    }
  }

  pub(crate) fn set_max_bytes(&mut self, max_bytes: u32) {
    self.max_bytes = max_bytes;
    while self.bytes_len() > self.max_bytes {
      self.pop_front();
    }
  }
}

impl<M> Default for AbstractHeaders<M> {
  #[inline]
  fn default() -> Self {
    Self {
      buffer: VecDeque::new(),
      elements_len: 0,
      first_idx: 0,
      max_bytes: DFLT_MAX_BYTES,
      metadata: VecDeque::new(),
    }
  }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Metadata<M> {
  is_activated: bool,
  misc: M,
  name_begin_idx: u32,
  sep_idx: u32,
  value_end_idx: u32,
}

#[cfg(test)]
mod tests {
  use crate::http::{abstract_headers::AbstractHeader, AbstractHeaders};

  #[test]
  fn elements_are_added_and_cleared() {
    let mut header = AbstractHeaders::default();
    header.push(0, b"abc", b"def");
    assert_eq!(
      header.get_by_name(b"abc"),
      Some(AbstractHeader {
        misc: &0,
        name_range: 0..3,
        name_bytes: "abc".as_bytes(),
        value_bytes: "def".as_bytes()
      })
    );
    assert_eq!(
      header.iter().next(),
      Some(AbstractHeader {
        misc: &0,
        name_range: 0..3,
        name_bytes: "abc".as_bytes(),
        value_bytes: "def".as_bytes()
      })
    );
    assert_eq!(header.elements_len(), 1);
    assert_eq!(header.bytes_len(), 6);
    header.clear();
    assert_eq!(header.get_by_name(b"abc"), None);
    assert_eq!(header.iter().next(), None);
    assert_eq!(header.elements_len(), 0);
    assert_eq!(header.bytes_len(), 0);
  }

  #[test]
  fn elements_are_added_and_removed() {
    let mut header = AbstractHeaders::default();

    header.push(0, b"abc", b"def");
    assert_eq!(
      header.get_by_name(b"abc"),
      Some(AbstractHeader {
        misc: &0,
        name_range: 0..3,
        name_bytes: "abc".as_bytes(),
        value_bytes: "def".as_bytes()
      })
    );
    assert_eq!(header.get_by_name(b"ghi"), None);
    assert_eq!(
      header.iter().nth(0),
      Some(AbstractHeader {
        misc: &0,
        name_range: 0..3,
        name_bytes: "abc".as_bytes(),
        value_bytes: "def".as_bytes()
      })
    );
    assert_eq!(header.iter().nth(1), None);
    assert_eq!(header.iter().nth(2), None);
    assert_eq!(header.elements_len(), 1);
    assert_eq!(header.bytes_len(), 6);
    header.push(1, b"ghi", b"jkl");
    assert_eq!(
      header.get_by_name(b"abc"),
      Some(AbstractHeader {
        misc: &0,
        name_range: 0..3,
        name_bytes: "abc".as_bytes(),
        value_bytes: "def".as_bytes()
      })
    );
    assert_eq!(
      header.get_by_name(b"ghi"),
      Some(AbstractHeader {
        misc: &1,
        name_range: 6..9,
        name_bytes: "ghi".as_bytes(),
        value_bytes: "jkl".as_bytes()
      })
    );
    assert_eq!(
      header.iter().nth(0),
      Some(AbstractHeader {
        misc: &0,
        name_range: 0..3,
        name_bytes: "abc".as_bytes(),
        value_bytes: "def".as_bytes()
      })
    );
    assert_eq!(
      header.iter().nth(1),
      Some(AbstractHeader {
        misc: &1,
        name_range: 6..9,
        name_bytes: "ghi".as_bytes(),
        value_bytes: "jkl".as_bytes()
      })
    );
    assert_eq!(header.iter().nth(2), None);
    assert_eq!(header.elements_len(), 2);
    assert_eq!(header.bytes_len(), 12);

    header.remove(&[b"123"]);
    assert_eq!(
      header.get_by_name(b"abc"),
      Some(AbstractHeader {
        misc: &0,
        name_range: 0..3,
        name_bytes: "abc".as_bytes(),
        value_bytes: "def".as_bytes()
      })
    );
    assert_eq!(
      header.get_by_name(b"ghi"),
      Some(AbstractHeader {
        misc: &1,
        name_range: 6..9,
        name_bytes: "ghi".as_bytes(),
        value_bytes: "jkl".as_bytes()
      })
    );
    assert_eq!(
      header.iter().nth(0),
      Some(AbstractHeader {
        misc: &0,
        name_range: 0..3,
        name_bytes: "abc".as_bytes(),
        value_bytes: "def".as_bytes()
      })
    );
    assert_eq!(
      header.iter().nth(1),
      Some(AbstractHeader {
        misc: &1,
        name_range: 6..9,
        name_bytes: "ghi".as_bytes(),
        value_bytes: "jkl".as_bytes()
      })
    );
    assert_eq!(header.iter().nth(2), None);
    assert_eq!(header.elements_len(), 2);
    assert_eq!(header.bytes_len(), 12);
    header.remove(&[b"abc"]);
    assert_eq!(header.get_by_name(b"abc"), None);
    assert_eq!(
      header.get_by_name(b"ghi"),
      Some(AbstractHeader {
        misc: &1,
        name_range: 6..9,
        name_bytes: "ghi".as_bytes(),
        value_bytes: "jkl".as_bytes()
      })
    );
    assert_eq!(
      header.iter().nth(0),
      Some(AbstractHeader {
        misc: &1,
        name_range: 6..9,
        name_bytes: "ghi".as_bytes(),
        value_bytes: "jkl".as_bytes()
      })
    );
    assert_eq!(header.iter().nth(1), None);
    assert_eq!(header.iter().nth(2), None);
    assert_eq!(header.elements_len(), 1);
    assert_eq!(header.bytes_len(), 12);
    header.remove(&[b"ghi"]);
    assert_eq!(header.get_by_name(b"abc"), None);
    assert_eq!(header.get_by_name(b"ghi"), None);
    assert_eq!(header.iter().nth(0), None);
    assert_eq!(header.iter().nth(1), None);
    assert_eq!(header.iter().nth(2), None);
    assert_eq!(header.elements_len(), 0);
    assert_eq!(header.bytes_len(), 12);
  }
}
