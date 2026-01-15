use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  misc::{
    SuffixWriterMut,
    counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write_iter},
  },
  tls::{SignatureScheme, TlsError, de::De, misc::u16_list},
};

#[derive(Debug)]
pub(crate) struct SignatureAlgorithmsCert {
  pub(crate) supported_groups: ArrayVectorU8<SignatureScheme, { SignatureScheme::len() }>,
}

impl<'de> Decode<'de, De> for SignatureAlgorithmsCert {
  #[inline]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let mut supported_groups = ArrayVectorU8::new();
    u16_list(&mut supported_groups, dw, TlsError::InvalidSignatureAlgorithmsCert)?;
    Ok(Self { supported_groups })
  }
}

impl Encode<De> for SignatureAlgorithmsCert {
  #[inline]
  fn encode(&self, sw: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      &self.supported_groups,
      None,
      sw,
      |el, local_sw| {
        let num: u16 = (*el).into();
        local_sw.extend_from_slice(&num.to_be_bytes())?;
        crate::Result::Ok(())
      },
    )?;
    Ok(())
  }
}
