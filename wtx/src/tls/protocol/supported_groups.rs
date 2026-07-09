use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorCopy,
  misc::counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write_iter},
  tls::{
    NamedGroup, TlsError, de::De, misc::u16_chunk, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Debug)]
pub(crate) struct SupportedGroups {
  pub(crate) supported_groups: ArrayVectorCopy<NamedGroup, { NamedGroup::len() }>,
}

impl<'de> Decode<'de, De> for SupportedGroups {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let mut supported_groups = ArrayVectorCopy::new();
    let bytes = u16_chunk(dw, TlsError::InvalidCipherSuite, |el| Ok(el.bytes()))?;
    for [b0, b1] in bytes.as_chunks::<2>().0 {
      if let Ok(elem) = NamedGroup::try_from(u16::from_be_bytes([*b0, *b1])) {
        supported_groups.push(elem)?;
      }
    }
    Ok(Self { supported_groups })
  }
}

impl Encode<De> for SupportedGroups {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      &self.supported_groups,
      None,
      ew,
      |el, local_ew| {
        local_ew.buffer().extend_from_copyable_slice(&u16::from(*el).to_be_bytes())?;
        crate::Result::Ok(())
      },
    )?;
    Ok(())
  }
}
