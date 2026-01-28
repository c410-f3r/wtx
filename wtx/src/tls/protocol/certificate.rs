// https://datatracker.ietf.org/doc/html/rfc8446#section-4.4.2

use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  misc::counter_writer::{
    CounterWriterBytesTy, CounterWriterIterTy, u8_write, u16_write, u24_write, u24_write_iter,
  },
  tls::{
    MAX_CERTIFICATES, TlsError,
    de::De,
    decode_wrapper::DecodeWrapper,
    encode_wrapper::EncodeWrapper,
    misc::{u8_chunk, u16_chunk, u24_chunk, u24_list},
    protocol::extension_ty::ExtensionTy,
  },
};

#[derive(Debug)]
pub struct Certificate<'any> {
  certificate_list: ArrayVectorU8<CertificateEntry<'any>, MAX_CERTIFICATES>,
  certificate_request_context: &'any [u8],
}

impl<'de> Decode<'de, De> for Certificate<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let certificate_request_context =
      u8_chunk(dw, TlsError::InvalidCertificate, |local_dw| Ok(local_dw.bytes()))?;
    let mut certificate_list = ArrayVectorU8::new();
    u24_list(&mut certificate_list, dw, TlsError::InvalidCertificate)?;
    Ok(Self { certificate_list, certificate_request_context })
  }
}

impl<'de> Encode<De> for Certificate<'de> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    u8_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      local_ew.buffer().extend_from_slice(self.certificate_request_context)?;
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

impl<'de> Decode<'de, De> for CertificateEntry<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let certificate_bytes =
      u24_chunk(dw, TlsError::InvalidCertificate, |local_dw| Ok(local_dw.bytes()))?;
    u16_chunk(dw, TlsError::InvalidCertificate, |local_dw| {
      while !local_dw.bytes().is_empty() {
        let extension_ty = ExtensionTy::decode(local_dw)?;
        match extension_ty {
          ExtensionTy::SignedCertificateTimestamp | ExtensionTy::StatusRequest => {
            return Err(TlsError::UnsupportedExtension.into());
          }
          _ => {
            return Err(TlsError::MismatchedExtension.into());
          }
        }
      }
      Ok(())
    })?;
    Ok(Self { certificate_bytes })
  }
}

impl<'de> Encode<De> for CertificateEntry<'de> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    u24_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      local_ew.buffer().extend_from_slice(self.certificate_bytes)?;
      crate::Result::Ok(())
    })?;
    u16_write(CounterWriterBytesTy::IgnoresLen, None, ew, |_| Ok(()))
  }
}
