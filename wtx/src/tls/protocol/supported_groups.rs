use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  misc::counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write_iter},
  tls::{NamedGroup, TlsError, de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper, misc::u16_list},
};

#[derive(Debug)]
pub(crate) struct SupportedGroups {
  pub(crate) supported_groups: ArrayVectorU8<NamedGroup, { NamedGroup::len() }>,
}

impl<'de> Decode<'de, De> for SupportedGroups {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let mut supported_groups = ArrayVectorU8::new();
    u16_list(&mut supported_groups, dw, TlsError::InvalidSupportedGroups)?;
    Ok(Self { supported_groups })
  }
}

impl Encode<De> for SupportedGroups {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      &self.supported_groups,
      None,
      ew,
      |el, local_sw| {
        local_sw.extend_from_slice(&u16::from(*el).to_be_bytes())?;
        crate::Result::Ok(())
      },
    )?;
    Ok(())
  }
}
