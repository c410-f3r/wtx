use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorCopy,
  misc::counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write_iter},
  tls::{
    NamedGroup, TlsError, de::De, misc::u16_chunk, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Clone, Debug)]
pub(crate) struct SupportedGroups {
  pub(crate) named_group_list: ArrayVectorCopy<NamedGroup, { NamedGroup::len() }>,
}

impl SupportedGroups {
  pub(crate) fn new(named_group_list: ArrayVectorCopy<NamedGroup, { NamedGroup::len() }>) -> Self {
    Self { named_group_list }
  }
}

impl<'de> Decode<'de, De> for SupportedGroups {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let mut named_group_list = ArrayVectorCopy::new();
    let bytes = u16_chunk(dw, TlsError::InvalidCipherSuite, |el| Ok(el.bytes()))?;
    for [b0, b1] in bytes.as_chunks::<2>().0 {
      if let Ok(elem) = NamedGroup::try_from(u16::from_be_bytes([*b0, *b1])) {
        named_group_list.push(elem)?;
      }
    }
    Ok(Self { named_group_list })
  }
}

impl Encode<De> for SupportedGroups {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      &self.named_group_list,
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
