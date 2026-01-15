use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  misc::{
    SuffixWriterMut,
    counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u8_write_iter},
  },
  tls::{TlsError, de::De, misc::u8_list, protocol::protocol_version::ProtocolVersion},
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
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let mut versions = ArrayVectorU8::new();
    u8_list(&mut versions, dw, TlsError::InvalidSupportedVersions)?;
    Ok(Self { versions })
  }
}

impl Encode<De> for SupportedVersions {
  #[inline]
  fn encode(&self, sw: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    u8_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      &self.versions,
      None,
      sw,
      |el, local_sw| {
        el.encode(local_sw)?;
        crate::Result::Ok(())
      },
    )?;
    Ok(())
  }
}
