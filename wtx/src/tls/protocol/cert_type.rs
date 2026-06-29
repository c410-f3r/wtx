use crate::{
  codec::{Decode, Encode},
  tls::{
    TlsCertificateTy, de::De, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

/// Can be `client_certificate_type` or `server_certificate_type`
#[derive(Clone, Debug, Default)]
pub(crate) struct CertType(pub(crate) TlsCertificateTy);

impl<'de> Decode<'de, De> for CertType {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    Ok(Self(TlsCertificateTy::decode(dw)?))
  }
}

impl Encode<De> for CertType {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    self.0.encode(ew)
  }
}
