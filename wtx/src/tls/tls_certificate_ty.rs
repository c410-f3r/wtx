use crate::{
  codec::{Decode, Encode},
  tls::{
    TlsError, de::De, tls_decode_wrapper::TlsDecodeWrapper, tls_encode_wrapper::TlsEncodeWrapper,
  },
};

create_enum! {
  /// Specifies the supported certificate type to the remote peer.
  #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
  pub enum TlsCertificateTy<u8> {
    /// DER-encoded bytes of the full X.509 certificate.
    #[default]
    X509 = (0),
    /// DER-encoded bytes of the `SubjectPublicKeyInfo` structure.
    RawPublicKey = (2),
  }
}

impl<'de> Decode<'de, De> for TlsCertificateTy {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let [b0, rest @ ..] = dw.bytes() else {
      return Err(TlsError::InvalidCertificateType.into());
    };
    *dw.bytes_mut() = rest;
    Self::try_from(*b0)
  }
}

impl Encode<De> for TlsCertificateTy {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().push(u8::from(*self))?;
    Ok(())
  }
}
