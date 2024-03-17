const MASK: u32 = 0b0111_1111_1111_1111_1111_1111_1111_1111;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StreamId(u32);

impl StreamId {
  pub(crate) const MAX: StreamId = StreamId(MASK);
  pub(crate) const ZERO: StreamId = StreamId(0);

  pub(crate) fn is_client_initiated(&self) -> bool {
    self.0 != 0 && self.0 % 2 == 1
  }

  pub(crate) fn is_not_zero(&self) -> bool {
    self.0 != 0
  }

  pub(crate) fn is_server_initiated(&self) -> bool {
    self.0 != 0 && self.0 % 2 == 0
  }

  pub(crate) fn is_zero(&self) -> bool {
    self.0 == 0
  }

  pub(crate) fn to_be_bytes(&self) -> [u8; 4] {
    self.0.to_be_bytes()
  }

  pub(crate) fn wrapping_add(&self, other: Self) -> Self {
    Self(self.0.wrapping_add(other.0))
  }
}

impl From<u32> for StreamId {
  #[inline]
  fn from(from: u32) -> Self {
    Self(from & MASK)
  }
}

impl PartialEq<u32> for StreamId {
  #[inline]
  fn eq(&self, other: &u32) -> bool {
    self.0 == *other
  }
}
