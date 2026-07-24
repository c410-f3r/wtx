use crate::{
  codec::{Decode, Encode},
  misc::counter_writer::{CounterWriterBytesTy, u16_write},
  tls::{
    SignatureScheme, TlsError, de::De, misc::u16_chunk, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Debug)]
pub(crate) struct CertificateVerify<'any> {
  algorithm: SignatureScheme,
  signature: &'any [u8],
}

impl<'any> CertificateVerify<'any> {
  pub(crate) fn new(algorithm: SignatureScheme, signature: &'any [u8]) -> Self {
    Self { algorithm, signature }
  }

  pub(crate) fn algorithm(&self) -> SignatureScheme {
    self.algorithm
  }

  pub(crate) fn signature(&self) -> &'any [u8] {
    self.signature
  }
}

impl<'de> Decode<'de, De> for CertificateVerify<'de> {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let algorithm = <SignatureScheme as Decode<'de, De>>::decode(dw)?;
    let signature = u16_chunk(dw, TlsError::InvalidCertificateVerify, |el| Ok(el.bytes()))?;
    Ok(Self { algorithm, signature })
  }
}

impl Encode<De> for CertificateVerify<'_> {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    <SignatureScheme as Encode<De>>::encode(&self.algorithm, ew)?;
    u16_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      local_ew.buffer().extend_from_copyable_slice(self.signature)?;
      Ok(())
    })
  }
}
