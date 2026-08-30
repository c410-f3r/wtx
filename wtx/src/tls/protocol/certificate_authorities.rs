// https://datatracker.ietf.org/doc/html/rfc8446#section-4.2.4

use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, SEQUENCE_TAG, SequenceBuffer},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper},
  collections::Vector,
  misc::{
    Lease,
    counter_writer::{CounterWriterBytesTy, u16_write},
  },
  tls::{
    TlsError,
    misc::u16_chunk,
    tls_cc::TlsCc,
    tls_cc_wrappers::{TlsDecodeWrapper, TlsEncodeWrapper},
  },
  x509::RelativeDistinguishedName,
};

#[derive(Clone, Debug, Default)]
pub(crate) struct CertificateAuthorities<B> {
  pub(crate) authorities: Vector<RelativeDistinguishedName<B>>,
}

impl<'de, B> Decode<'de, TlsCc> for CertificateAuthorities<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    u16_chunk(dw, TlsError::InvalidCertificateAuthorities, |local_dw| {
      let mut authorities = Vector::new();
      while !local_dw.bytes().is_empty() {
        u16_chunk(local_dw, TlsError::InvalidCertificateAuthorities, |dn_dw| {
          let mut asn1_dw = DecodeWrapper::new(dn_dw.bytes(), Asn1DecodeWrapperAux::default());
          let (instance, _) = SequenceBuffer::<Vector<_>>::decode(&mut asn1_dw, SEQUENCE_TAG)?;
          for rdn in instance.0 {
            authorities.push(rdn)?;
          }
          Ok(())
        })?;
      }

      Ok(Self { authorities })
    })
  }
}

impl<B> Encode<TlsCc> for CertificateAuthorities<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    u16_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      for name in &self.authorities {
        u16_write(CounterWriterBytesTy::IgnoresLen, None, local_ew, |dn_ew| {
          name.encode(&mut EncodeWrapper::new(dn_ew.buffer(), Asn1EncodeWrapperAux::default()))
        })?;
      }
      Ok(())
    })
  }
}
