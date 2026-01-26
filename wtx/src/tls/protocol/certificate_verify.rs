use crate::{
  de::{Decode, Encode},
  misc::counter_writer::{CounterWriterBytesTy, u16_write},
  tls::{
    TlsError, de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper,
    misc::u16_chunk, protocol::signature_scheme::SignatureScheme,
  },
};

#[derive(Debug)]
pub struct CertificateVerify<'any> {
  algorithm: SignatureScheme,
  signature: &'any [u8],
}

impl<'de> Decode<'de, De> for CertificateVerify<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let algorithm = SignatureScheme::decode(dw)?;
    let signature = u16_chunk(dw, TlsError::InvalidCertificateVerify, |el| Ok(el.bytes()))?;
    Ok(Self { algorithm, signature })
  }
}

impl<'de> Encode<De> for CertificateVerify<'de> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    self.algorithm.encode(ew)?;
    u16_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      local_ew.buffer().extend_from_slice(self.signature)?;
      Ok(())
    })
  }
}
