use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorCopy,
  misc::counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u8_write_iter},
  tls::{
    TlsError, de::De, misc::u8_list, protocol::protocol_version::ProtocolVersion,
    tls_decode_wrapper::TlsDecodeWrapper, tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Debug)]
pub(crate) struct SupportedVersions {
  pub(crate) versions: ArrayVectorCopy<ProtocolVersion, 1>,
}

impl SupportedVersions {
  pub(crate) fn new(versions: ArrayVectorCopy<ProtocolVersion, 1>) -> Self {
    Self { versions }
  }
}

impl<'de> Decode<'de, De> for SupportedVersions {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let mut versions = ArrayVectorCopy::new();
    u8_list(&mut versions, dw, TlsError::InvalidSupportedVersions)?;
    Ok(Self { versions })
  }
}

impl Encode<De> for SupportedVersions {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
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
