use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorCopy,
  crypto::SignatureTy,
  misc::counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write_iter},
  tls::{
    TlsError, de::De, misc::u16_list, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Clone, Debug)]
pub(crate) struct SignatureAlgorithmsCert {
  pub(crate) supported_groups: ArrayVectorCopy<SignatureTy, { SignatureTy::len() }>,
}

impl SignatureAlgorithmsCert {
  pub(crate) fn new(
    supported_groups: ArrayVectorCopy<SignatureTy, { SignatureTy::len() }>,
  ) -> Self {
    Self { supported_groups }
  }
}

impl<'de> Decode<'de, De> for SignatureAlgorithmsCert {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let mut supported_groups = ArrayVectorCopy::new();
    u16_list(&mut supported_groups, dw, TlsError::InvalidSignatureAlgorithmsCert)?;
    Ok(Self { supported_groups })
  }
}

impl Encode<De> for SignatureAlgorithmsCert {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      &self.supported_groups,
      None,
      ew,
      |el, local_ew| {
        let num: u16 = (*el).into();
        local_ew.buffer().extend_from_copyable_slice(&num.to_be_bytes())?;
        crate::Result::Ok(())
      },
    )?;
    Ok(())
  }
}
