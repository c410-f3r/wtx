/// Represents the absence of a body. Mostly used in HTTP GET requests.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct EmptyBody;

#[cfg(feature = "quick-protobuf")]
mod quick_protobuf {
  use crate::codec::EmptyBody;
  use quick_protobuf::{BytesReader, MessageRead, Result};

  impl<'de> MessageRead<'de> for EmptyBody {
    #[inline]
    fn from_reader(_: &mut BytesReader, _: &'de [u8]) -> Result<Self> {
      Ok(EmptyBody)
    }
  }
}
