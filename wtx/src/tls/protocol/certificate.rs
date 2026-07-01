// https://datatracker.ietf.org/doc/html/rfc8446#section-4.4.2

use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorU8,
  misc::counter_writer::{
    CounterWriterBytesTy, CounterWriterIterTy, u8_write, u16_write, u24_write, u24_write_iter,
  },
  tls::{
    MAX_CERTIFICATES, TlsError,
    de::De,
    misc::{u8_chunk, u16_chunk, u24_chunk, u24_list},
    protocol::extension_ty::ExtensionTy,
    tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Debug)]
pub(crate) struct Certificate<'any> {
  certificate_list: ArrayVectorU8<CertificateEntry<'any>, MAX_CERTIFICATES>,
  certificate_request_context: &'any [u8],
}

impl<'any> Certificate<'any> {
  pub(crate) fn new(
    certificate_list: ArrayVectorU8<CertificateEntry<'any>, MAX_CERTIFICATES>,
    certificate_request_context: &'any [u8],
  ) -> Self {
    Self { certificate_list, certificate_request_context }
  }

  pub(crate) fn certificate_list(
    &self,
  ) -> &ArrayVectorU8<CertificateEntry<'any>, MAX_CERTIFICATES> {
    &self.certificate_list
  }
}

impl<'de> Decode<'de, De> for Certificate<'de> {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let err = TlsError::InvalidCertificate;
    let certificate_request_context = u8_chunk(dw, err, |local_dw| Ok(local_dw.bytes()))?;
    let mut certificate_list = ArrayVectorU8::new();
    u24_list(&mut certificate_list, dw, err)?;
    Ok(Self { certificate_list, certificate_request_context })
  }
}

impl Encode<De> for Certificate<'_> {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    u8_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      local_ew.buffer().extend_from_copyable_slice(self.certificate_request_context)?;
      crate::Result::Ok(())
    })?;
    u24_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      &self.certificate_list,
      None,
      ew,
      |el, local_ew| {
        el.encode(local_ew)?;
        Ok(())
      },
    )
  }
}

#[derive(Debug)]
pub(crate) struct CertificateEntry<'any> {
  certificate_bytes: &'any [u8],
}

impl<'any> CertificateEntry<'any> {
  pub(crate) fn new(certificate_bytes: &'any [u8]) -> Self {
    Self { certificate_bytes }
  }

  pub(crate) const fn certificate_bytes(&self) -> &'any [u8] {
    self.certificate_bytes
  }
}

impl<'de> Decode<'de, De> for CertificateEntry<'de> {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let certificate_bytes =
      u24_chunk(dw, TlsError::InvalidCertificate, |local_dw| Ok(local_dw.bytes()))?;
    u16_chunk(dw, TlsError::InvalidCertificate, |local_dw| {
      let extension_ty = ExtensionTy::decode(local_dw)?;
      match extension_ty {
        ExtensionTy::SignedCertificateTimestamp | ExtensionTy::StatusRequest => {
          Err(TlsError::UnsupportedExtension.into())
        }
        ExtensionTy::ApplicationLayerProtocolNegotiation
        | ExtensionTy::CertificateAuthorities
        | ExtensionTy::ClientCertificateType
        | ExtensionTy::Cookie
        | ExtensionTy::EarlyData
        | ExtensionTy::Heartbeat
        | ExtensionTy::KeyShare
        | ExtensionTy::MaxFragmentLength
        | ExtensionTy::OidFilters
        | ExtensionTy::Padding
        | ExtensionTy::PostHandshakeAuth
        | ExtensionTy::PreSharedKey
        | ExtensionTy::PskKeyExchangeModes
        | ExtensionTy::ServerCertificateType
        | ExtensionTy::ServerName
        | ExtensionTy::SignatureAlgorithms
        | ExtensionTy::SignatureAlgorithmsCert
        | ExtensionTy::SupportedGroups
        | ExtensionTy::SupportedVersions
        | ExtensionTy::UseSrtp => Err(TlsError::MismatchedExtension.into()),
      }
    })?;
    Ok(Self { certificate_bytes })
  }
}

impl Encode<De> for CertificateEntry<'_> {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    u24_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      local_ew.buffer().extend_from_copyable_slice(self.certificate_bytes)?;
      crate::Result::Ok(())
    })?;
    u16_write(CounterWriterBytesTy::IgnoresLen, None, ew, |_| Ok(()))
  }
}
