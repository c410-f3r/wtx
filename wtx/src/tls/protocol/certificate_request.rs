// https://datatracker.ietf.org/doc/html/rfc8446#section-4.3.2

use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  misc::counter_writer::{CounterWriterBytesTy, u8_write, u16_write},
  tls::{
    TlsError,
    de::De,
    decode_wrapper::DecodeWrapper,
    encode_wrapper::EncodeWrapper,
    misc::{duplicated_error, u8_chunk, u16_chunk},
    protocol::{
      extension::Extension, extension_ty::ExtensionTy, signature_algorithms::SignatureAlgorithms,
      signature_scheme::SignatureScheme,
    },
  },
};

#[derive(Debug)]
pub struct CertificateRequest<'any> {
  pub(crate) certificate_request_context: &'any [u8],
  pub(crate) signature_algorithms: ArrayVectorU8<SignatureScheme, { SignatureScheme::len() }>,
}

impl<'de> Decode<'de, De> for CertificateRequest<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let certificate_request_context =
      u8_chunk(dw, TlsError::InvalidCertificateRequest, |local_dw| Ok(local_dw.bytes()))?;
    let mut signature_algorithms = ArrayVectorU8::new();
    u16_chunk(dw, TlsError::InvalidCertificateRequest, |local_dw| {
      while !local_dw.bytes().is_empty() {
        let extension_ty = {
          let tmp_bytes = &mut *local_dw;
          ExtensionTy::decode(tmp_bytes)?
        };
        match extension_ty {
          ExtensionTy::SignatureAlgorithms => {
            duplicated_error(!signature_algorithms.is_empty())?;
            signature_algorithms =
              Extension::<SignatureAlgorithms>::decode(local_dw)?.into_data().signature_schemes;
          }
          ExtensionTy::CertificateAuthorities
          | ExtensionTy::OidFilters
          | ExtensionTy::SignedCertificateTimestamp
          | ExtensionTy::SignatureAlgorithmsCert
          | ExtensionTy::StatusRequest => {
            return Err(TlsError::UnsupportedExtension.into());
          }
          _ => {
            return Err(TlsError::MismatchedExtension.into());
          }
        }
      }
      Ok(())
    })?;

    Ok(Self { certificate_request_context, signature_algorithms })
  }
}

impl<'any> Encode<De> for CertificateRequest<'any> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    u8_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      local_ew.buffer().extend_from_slice(self.certificate_request_context)?;
      crate::Result::Ok(())
    })?;
    u16_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      Extension::new(
        ExtensionTy::SignatureAlgorithms,
        SignatureAlgorithms {
          signature_schemes: ArrayVectorU8::from_iterator(
            self.signature_algorithms.iter().copied(),
          )?,
        },
      )
      .encode(local_ew)?;
      crate::Result::Ok(())
    })?;
    Ok(())
  }
}
