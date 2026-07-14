use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorCopy,
  misc::counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u8_write_iter},
  tls::{
    TlsError, de::De, misc::u8_chunk, protocol::protocol_version::ProtocolVersion,
    tls_decode_wrapper::TlsDecodeWrapper, tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Debug)]
pub(crate) struct SupportedVersionsClient {
  pub(crate) versions: ArrayVectorCopy<ProtocolVersion, 8>,
}

impl SupportedVersionsClient {
  pub(crate) fn new(versions: ArrayVectorCopy<ProtocolVersion, 8>) -> Self {
    Self { versions }
  }
}

impl<'de> Decode<'de, De> for SupportedVersionsClient {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let mut versions = ArrayVectorCopy::new();
    let bytes = u8_chunk(dw, TlsError::InvalidSupportedVersions, |el| Ok(el.bytes()))?;
    for [b0, b1] in bytes.as_chunks::<2>().0 {
      if let Ok(elem) = ProtocolVersion::try_from(u16::from_be_bytes([*b0, *b1])) {
        versions.push(elem)?;
      }
    }
    Ok(Self { versions })
  }
}

impl Encode<De> for SupportedVersionsClient {
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

#[derive(Debug)]
pub(crate) struct SupportedVersionsServer {
  pub(crate) selected_version: ProtocolVersion,
}

impl SupportedVersionsServer {
  pub(crate) fn new(selected_version: ProtocolVersion) -> Self {
    Self { selected_version }
  }
}

impl<'de> Decode<'de, De> for SupportedVersionsServer {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    Ok(Self { selected_version: ProtocolVersion::decode(dw)? })
  }
}

impl Encode<De> for SupportedVersionsServer {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    self.selected_version.encode(ew)
  }
}
