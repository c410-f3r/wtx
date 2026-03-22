use crate::{codec::CodecController, collection::Vector};

/// A data structure that can be deserialized from a data format.
pub trait Decode<'de, CC>: Sized
where
  CC: CodecController,
{
  /// Performs the conversion.
  fn decode(dw: &mut CC::DecodeWrapper<'de, '_, '_>) -> Result<Self, CC::Error>;
}

impl Decode<'_, ()> for &str {
  #[inline]
  fn decode(_: &mut ()) -> Result<Self, crate::Error> {
    Ok("")
  }
}

impl Decode<'_, ()> for u32 {
  #[inline]
  fn decode(_: &mut ()) -> Result<Self, crate::Error> {
    Ok(0)
  }
}

impl Decode<'_, ()> for u64 {
  #[inline]
  fn decode(_: &mut ()) -> Result<Self, crate::Error> {
    Ok(0)
  }
}

/// Decode sequence
pub trait DecodeSeq<'de, CC>: Decode<'de, CC>
where
  CC: CodecController,
{
  /// Decodes a sequence of itself into a buffer
  fn decode_seq(
    buffer: &mut Vector<Self>,
    dw: &mut CC::DecodeWrapper<'de, '_, '_>,
  ) -> Result<(), CC::Error>;
}

impl DecodeSeq<'_, ()> for &str {
  #[inline]
  fn decode_seq(_: &mut Vector<Self>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

impl DecodeSeq<'_, ()> for u32 {
  #[inline]
  fn decode_seq(_: &mut Vector<Self>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

impl DecodeSeq<'_, ()> for u64 {
  #[inline]
  fn decode_seq(_: &mut Vector<Self>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}
