use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorCopy,
  misc::counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write_iter},
  tls::{
    SignatureScheme, TlsError, de::De, misc::u16_chunk, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Clone, Debug)]
pub(crate) struct SignatureAlgorithmsCert {
  pub(crate) signature_schemes: ArrayVectorCopy<SignatureScheme, { SignatureScheme::len() }>,
}

impl SignatureAlgorithmsCert {
  pub(crate) fn new(
    signature_schemes: ArrayVectorCopy<SignatureScheme, { SignatureScheme::len() }>,
  ) -> Self {
    Self { signature_schemes }
  }
}

impl<'de> Decode<'de, De> for SignatureAlgorithmsCert {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let mut signature_schemes = ArrayVectorCopy::new();
    let bytes = u16_chunk(dw, TlsError::InvalidCipherSuite, |el| Ok(el.bytes()))?;
    for [b0, b1] in bytes.as_chunks::<2>().0 {
      if let Ok(elem) = SignatureScheme::try_from(u16::from_be_bytes([*b0, *b1])) {
        signature_schemes.push(elem)?;
      }
    }
    Ok(Self { signature_schemes })
  }
}

impl Encode<De> for SignatureAlgorithmsCert {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      &self.signature_schemes,
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

impl Default for SignatureAlgorithmsCert {
  #[inline]
  fn default() -> Self {
    Self::new(ArrayVectorCopy::from_array(SignatureScheme::PRIORITY))
  }
}
