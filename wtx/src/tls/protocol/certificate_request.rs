// https://datatracker.ietf.org/doc/html/rfc8446#section-4.3.2

use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorCopy,
  misc::counter_writer::{CounterWriterBytesTy, u8_write, u16_write},
  tls::{
    TlsError,
    de::De,
    misc::{duplicated_error, u8_chunk, u16_chunk},
    protocol::{
      extension::Extension, extension_ty::ExtensionTy, signature_algorithms::SignatureAlgorithms,
    },
    tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Debug, PartialEq)]
#[expect(dead_code, reason = "Future-proof mTLS")]
pub(crate) struct CertificateRequest {
  pub(crate) certificate_request_context: ArrayVectorCopy<u8, 32>,
  pub(crate) signature_algorithms: SignatureAlgorithms,
}

impl<'de> Decode<'de, De> for CertificateRequest {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let err = TlsError::InvalidCertificateRequest;
    let certificate_request_context = u8_chunk(dw, err, |el| Ok(el.bytes()))?.try_into()?;
    let mut signature_algorithms = None;
    u16_chunk(dw, err, |local_dw| {
      while !local_dw.bytes().is_empty() {
        let extension_ty = ExtensionTy::decode(local_dw)?;
        u16_chunk(local_dw, err, |local_local_dw| {
          manage_extension(local_local_dw, extension_ty, &mut signature_algorithms)
        })?;
      }
      Ok(())
    })?;
    Ok(Self { certificate_request_context, signature_algorithms: signature_algorithms.ok_or(err)? })
  }
}

impl Encode<De> for CertificateRequest {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    u8_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      local_ew.buffer().extend_from_copyable_slice(&self.certificate_request_context)?;
      crate::Result::Ok(())
    })?;
    u16_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      Extension::new(
        ExtensionTy::SignatureAlgorithms,
        SignatureAlgorithms {
          signature_schemes: ArrayVectorCopy::from_iterator(
            self.signature_algorithms.signature_schemes.iter().copied(),
          )?,
        },
      )
      .encode(local_ew)?;
      crate::Result::Ok(())
    })?;
    Ok(())
  }
}

#[expect(dead_code, reason = "Future-proof mTLS")]
#[inline]
fn manage_extension(
  dw: &mut TlsDecodeWrapper<'_>,
  extension_ty: ExtensionTy,
  signature_algorithms: &mut Option<SignatureAlgorithms>,
) -> crate::Result<()> {
  match extension_ty {
    ExtensionTy::SignatureAlgorithms => {
      duplicated_error(signature_algorithms.is_some())?;
      *signature_algorithms = Some(SignatureAlgorithms::decode(dw)?);
    }
    ExtensionTy::CertificateAuthorities
    | ExtensionTy::OidFilters
    | ExtensionTy::SignedCertificateTimestamp
    | ExtensionTy::SignatureAlgorithmsCert
    | ExtensionTy::StatusRequest => {}
    ExtensionTy::ApplicationLayerProtocolNegotiation
    | ExtensionTy::ClientCertificateType
    | ExtensionTy::Cookie
    | ExtensionTy::EarlyData
    | ExtensionTy::Heartbeat
    | ExtensionTy::KeyShare
    | ExtensionTy::MaxFragmentLength
    | ExtensionTy::Padding
    | ExtensionTy::PostHandshakeAuth
    | ExtensionTy::PreSharedKey
    | ExtensionTy::PskKeyExchangeModes
    | ExtensionTy::ServerCertificateType
    | ExtensionTy::ServerName
    | ExtensionTy::SupportedGroups
    | ExtensionTy::SupportedVersions
    | ExtensionTy::UseSrtp => {
      return Err(TlsError::MismatchedExtension.into());
    }
  }
  Ok(())
}
