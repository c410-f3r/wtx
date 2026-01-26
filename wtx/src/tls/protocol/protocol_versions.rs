use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  misc::counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u8_write_iter},
  tls::{
    TlsError, de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper, misc::u8_list,
    protocol::protocol_version::ProtocolVersion,
  },
};

#[derive(Debug)]
pub(crate) struct SupportedVersions {
  pub(crate) versions: ArrayVectorU8<ProtocolVersion, 1>,
}

impl SupportedVersions {
  pub(crate) fn new(versions: ArrayVectorU8<ProtocolVersion, 1>) -> Self {
    Self { versions }
  }
}

impl<'de> Decode<'de, De> for SupportedVersions {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let mut versions = ArrayVectorU8::new();
    u8_list(&mut versions, dw, TlsError::InvalidSupportedVersions)?;
    Ok(Self { versions })
  }
}

impl Encode<De> for SupportedVersions {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    u8_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      &self.versions,
      None,
      ew,
      |el, local_ew| {
        el.encode(local_ew)?;
        crate::Result::Ok(())
      },
    )?;
    Ok(())
  }
}
