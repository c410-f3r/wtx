use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorU8,
  misc::counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write_iter},
  tls::{
    NamedGroup, TlsError, de::De, misc::u16_list, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Debug)]
pub(crate) struct SupportedGroups {
  pub(crate) supported_groups: ArrayVectorU8<NamedGroup, { NamedGroup::len() }>,
}

impl<'de> Decode<'de, De> for SupportedGroups {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let mut supported_groups = ArrayVectorU8::new();
    u16_list(&mut supported_groups, dw, TlsError::InvalidSupportedGroups)?;
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
        local_ew.buffer().inner_mut().extend_from_copyable_slice(&u16::from(*el).to_be_bytes())?;
        crate::Result::Ok(())
      },
    )?;
    Ok(())
  }
}
