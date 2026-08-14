use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorCopy,
  misc::counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write_iter},
  tls::{
    SignatureScheme, TlsError, de::De, misc::u16_chunk, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

/// Applies to signatures in `CertificateVerify` messages.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SignatureAlgorithms {
  /// List of [`SignatureScheme`].
  pub signature_schemes: ArrayVectorCopy<SignatureScheme, { SignatureScheme::len() }>,
}

impl SignatureAlgorithms {
  pub(crate) const fn new(
    signature_schemes: ArrayVectorCopy<SignatureScheme, { SignatureScheme::len() }>,
  ) -> Self {
    Self { signature_schemes }
  }
}

impl<'de> Decode<'de, De> for SignatureAlgorithms {
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

impl Encode<De> for SignatureAlgorithms {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    let iter = &self.signature_schemes;
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      iter,
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

impl Default for SignatureAlgorithms {
  #[inline]
  fn default() -> Self {
    Self::new(ArrayVectorCopy::from_array(SignatureScheme::PRIORITY))
  }
}
