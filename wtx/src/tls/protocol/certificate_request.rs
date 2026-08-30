// https://datatracker.ietf.org/doc/html/rfc9846#section-4.3.2

use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorCopy,
  misc::counter_writer::{CounterWriterBytesTy, u8_write, u16_write},
  tls::{
    AlertDescription, TlsError,
    misc::{decode_extension_ty, u8_chunk, u16_chunk},
    protocol::{
      extension::Extension, extension_ty::ExtensionTy, signature_algorithms::SignatureAlgorithms,
    },
    tls_cc::TlsCc,
    tls_cc_wrappers::{TlsDecodeWrapper, TlsEncodeWrapper},
  },
};

#[derive(Debug, PartialEq)]
pub(crate) struct CertificateRequest {
  pub(crate) certificate_request_context: ArrayVectorCopy<u8, 32>,
  pub(crate) signature_algorithms: SignatureAlgorithms,
}

impl<'de> Decode<'de, TlsCc> for CertificateRequest {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let err = TlsError::InvalidCertificateRequest;
    let crc: ArrayVectorCopy<u8, 32> = u8_chunk(dw, err, |el| Ok(el.bytes()))?.try_into()?;
    if !crc.is_empty() {
      return Err(TlsError::InvalidCertificateRequest.into());
    }
    let mut signature_algorithms = None;
    u16_chunk(dw, err, |local_dw| {
      let mut seen_unknowns = ArrayVectorCopy::new();
      while !local_dw.bytes().is_empty() {
        let Some(extension_ty) = decode_extension_ty(local_dw, err, &mut seen_unknowns)? else {
          continue;
        };
        u16_chunk(local_dw, err, |local_local_dw| {
          manage_extension(local_local_dw, extension_ty, &mut signature_algorithms)
        })?;
      }
      Ok(())
    })?;
    Ok(Self {
      certificate_request_context: crc,
      signature_algorithms: signature_algorithms.ok_or(err)?,
    })
  }
}

impl Encode<TlsCc> for CertificateRequest {
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

#[inline]
fn duplicated_error(is_some: bool) -> crate::Result<()> {
  if is_some {
    return Err(TlsError::DuplicatedCertificateRequestParameters.into());
  }
  Ok(())
}

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
      return Err(crate::Error::TlsErrorReply(
        TlsError::MismatchedExtension,
        AlertDescription::BadRecordMac,
      ));
    }
  }
  Ok(())
}
