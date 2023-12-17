use crate::http2::RawHeader;
use alloc::collections::VecDeque;

#[derive(Debug, Eq, PartialEq)]
enum Index<'data> {
  // The header is already fully indexed
  Indexed(usize, RawHeader<'data>),
  // The full header has been inserted into the table.
  Inserted(usize),
  // Only the value has been inserted (hpack table idx, slots idx)
  InsertedValue(usize, usize),
  // The name is indexed, but not the value
  Name(usize, RawHeader<'data>),
  // The header is not indexed by this table
  NotIndexed(RawHeader<'data>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SizeUpdate {
  One(usize),
  Two(usize, usize), // min, max
}

#[derive(Debug, Eq, PartialEq)]
pub struct Encoder<'data> {
  table: EncoderTable<'data>,
  size_update: Option<SizeUpdate>,
}

impl<'data> Encoder<'data> {
  fn new(max_size: usize) -> Self {
    Self { table: EncoderTable::new(max_size), size_update: None }
  }

  pub fn encode(&mut self, headers: impl IntoIterator<Item = RawHeader<'data>>, to: &mut Vec<u8>) {
    todo!()
  }

  fn encode_size_updates(&mut self, to: &mut Vec<u8>) {
    todo!()
  }

  fn encode_header(&mut self, index: &Index, to: &mut Vec<u8>) {
    todo!()
  }

  fn encode_header_without_name(&mut self, last: &Index, value: &[u8], to: &mut Vec<u8>) {
    todo!()
  }

  fn update_max_size(&mut self, val: usize) {
    match self.size_update {
      Some(SizeUpdate::One(old)) => {
        if val > old {
          if old > self.table.max_size {
            self.size_update = Some(SizeUpdate::One(val));
          } else {
            self.size_update = Some(SizeUpdate::Two(old, val));
          }
        } else {
          self.size_update = Some(SizeUpdate::One(val));
        }
      }
      Some(SizeUpdate::Two(min, _)) => {
        if val < min {
          self.size_update = Some(SizeUpdate::One(val));
        } else {
          self.size_update = Some(SizeUpdate::Two(min, val));
        }
      }
      None => {
        if val != self.table.max_size {
          // Don't bother writing a frame if the value already matches
          // the table's max size.
          self.size_update = Some(SizeUpdate::One(val));
        }
      }
    }
  }
}

#[derive(Debug, Eq, PartialEq)]
struct EncoderTable<'data> {
  indices: Vec<Option<Pos>>,
  inserted: usize,
  mask: usize,
  max_size: usize,
  size: usize,
  slots: VecDeque<Slot<'data>>,
}

impl<'data> EncoderTable<'data> {
  fn new(max_size: usize) -> Self {
    Self { indices: Vec::new(), inserted: 0, mask: 0, max_size, size: 0, slots: VecDeque::new() }
  }

  fn index(&mut self, header: RawHeader) -> Index {
    todo!()
  }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Pos {
  hash: usize,
  index: usize,
}

#[derive(Debug, Eq, PartialEq)]
struct Slot<'data> {
  hash: usize,
  header: RawHeader<'data>,
  next: Option<usize>,
}
