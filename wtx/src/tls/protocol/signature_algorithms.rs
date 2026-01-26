use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  misc::counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write_iter},
  tls::{
    TlsError, de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper, misc::u16_list, protocol::signature_scheme::SignatureScheme
  },
};

#[derive(Debug)]
pub(crate) struct SignatureAlgorithms {
  pub(crate) signature_schemes: ArrayVectorU8<SignatureScheme, { SignatureScheme::len() }>,
}

impl<'de> Decode<'de, De> for SignatureAlgorithms {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let mut signature_schemes = ArrayVectorU8::new();
    u16_list(&mut signature_schemes, dw, TlsError::InvalidSignatureAlgorithms)?;
    Ok(Self { signature_schemes })
  }
}

impl Encode<De> for SignatureAlgorithms {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    let iter = &self.signature_schemes;
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      iter,
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
