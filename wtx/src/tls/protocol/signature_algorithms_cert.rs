use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  misc::counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write_iter},
  tls::{
    TlsError, de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper, misc::u16_list, protocol::signature_scheme::SignatureScheme
  },
};

#[derive(Debug)]
pub(crate) struct SignatureAlgorithmsCert {
  pub(crate) supported_groups: ArrayVectorU8<SignatureScheme, { SignatureScheme::len() }>,
}

impl<'de> Decode<'de, De> for SignatureAlgorithmsCert {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let mut supported_groups = ArrayVectorU8::new();
    u16_list(&mut supported_groups, dw, TlsError::InvalidSignatureAlgorithmsCert)?;
    Ok(Self { supported_groups })
  }
}

impl Encode<De> for SignatureAlgorithmsCert {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      &self.supported_groups,
      None,
      ew,
      |el, local_sw| {
        let num: u16 = (*el).into();
        local_sw.extend_from_slice(&num.to_be_bytes())?;
        crate::Result::Ok(())
      },
    )?;
    Ok(())
  }
}
