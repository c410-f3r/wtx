use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  misc::{
    SuffixWriterMut,
    counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write_iter},
  },
  tls::{NamedGroup, TlsError, de::De, misc::u16_list},
};

#[derive(Debug)]
pub(crate) struct SupportedGroups {
  pub(crate) supported_groups: ArrayVectorU8<NamedGroup, { NamedGroup::len() }>,
}

impl<'de> Decode<'de, De> for SupportedGroups {
  #[inline]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let mut supported_groups = ArrayVectorU8::new();
    u16_list(&mut supported_groups, dw, TlsError::InvalidSupportedGroups)?;
    Ok(Self { supported_groups })
  }
}

impl Encode<De> for SupportedGroups {
  #[inline]
  fn encode(&self, sw: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      &self.supported_groups,
      None,
      sw,
      |el, local_sw| {
        local_sw.extend_from_slice(&u16::from(*el).to_be_bytes())?;
        crate::Result::Ok(())
      },
    )?;
    Ok(())
  }
}
