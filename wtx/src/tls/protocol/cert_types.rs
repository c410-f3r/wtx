use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorCopy,
  misc::counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u8_write_iter},
  tls::{
    TlsCertificateTy, TlsError, de::De, misc::u8_list, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

/// Can be `client_certificate_type` or `server_certificate_type`
#[derive(Clone, Debug)]
pub(crate) struct CertTypes(pub(crate) ArrayVectorCopy<TlsCertificateTy, 2>);

impl<'de> Decode<'de, De> for CertTypes {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let mut rslt = ArrayVectorCopy::new();
    u8_list(&mut rslt, dw, TlsError::InvalidCertificateType)?;
    Ok(Self(rslt))
  }
}

#[expect(clippy::redundant_closure_for_method_calls, reason = "false-positive")]
impl Encode<De> for CertTypes {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    u8_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      &self.0,
      None,
      ew,
      |el, local_ew| el.encode(local_ew),
    )
  }
}

impl Default for CertTypes {
  #[inline]
  fn default() -> Self {
    Self(ArrayVectorCopy::from_array([TlsCertificateTy::X509]))
  }
}
