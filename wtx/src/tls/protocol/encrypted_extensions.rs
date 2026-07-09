// https://datatracker.ietf.org/doc/html/rfc8446#section-4.3.1

use crate::{
  codec::{Decode, Encode},
  misc::counter_writer::{CounterWriterBytesTy, u16_write},
  tls::{
    MaxFragmentLength, TlsError,
    de::De,
    misc::{duplicated_error, u16_chunk},
    protocol::{
      alpn::Alpn, extension::Extension, extension_ty::ExtensionTy,
      server_name_list::ServerNameList, supported_groups::SupportedGroups,
    },
    tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Debug)]
pub(crate) struct EncryptedExtensions {
  alpn: Option<Alpn>,
  max_fragment_length: Option<MaxFragmentLength>,
  server_name: Option<ServerNameList>,
  supported_groups: Option<SupportedGroups>,
}

impl EncryptedExtensions {
  pub(crate) fn new(
    alpn: Option<Alpn>,
    max_fragment_length: Option<MaxFragmentLength>,
    server_name: Option<ServerNameList>,
    supported_groups: Option<SupportedGroups>,
  ) -> Self {
    Self { alpn, max_fragment_length, server_name, supported_groups }
  }

  pub(crate) const fn max_fragment_length(&self) -> Option<MaxFragmentLength> {
    self.max_fragment_length
  }
}

impl<'de> Decode<'de, De> for EncryptedExtensions {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let err = TlsError::InvalidEncryptedExtensions;
    let mut alpn = None;
    let mut max_fragment_length = None;
    let mut server_name = None;
    let mut supported_groups = None;
    u16_chunk(dw, err, |local_dw| {
      while !local_dw.bytes().is_empty() {
        let extension_ty = ExtensionTy::decode(local_dw)?;
        u16_chunk(local_dw, err, |local_local_dw| {
          manage_extension(
            &mut alpn,
            local_local_dw,
            extension_ty,
            &mut max_fragment_length,
            &mut server_name,
            &mut supported_groups,
          )
        })?;
      }
      Ok(())
    })?;
    Ok(Self { alpn, max_fragment_length, server_name, supported_groups })
  }
}

impl Encode<De> for EncryptedExtensions {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    let Self { alpn, max_fragment_length, server_name, supported_groups } = self;
    u16_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      if let Some(el) = alpn {
        Extension::new(ExtensionTy::ApplicationLayerProtocolNegotiation, el).encode(local_ew)?;
      }
      if let Some(el) = max_fragment_length {
        Extension::new(ExtensionTy::MaxFragmentLength, el).encode(local_ew)?;
      }
      if let Some(el) = server_name {
        Extension::new(ExtensionTy::ServerName, el).encode(local_ew)?;
      }
      if let Some(el) = supported_groups {
        Extension::new(ExtensionTy::SupportedGroups, el).encode(local_ew)?;
      }
      Ok(())
    })
  }
}

#[inline]
fn manage_extension(
  alpn: &mut Option<Alpn>,
  dw: &mut TlsDecodeWrapper<'_>,
  extension_ty: ExtensionTy,
  max_fragment_length: &mut Option<MaxFragmentLength>,
  server_name: &mut Option<ServerNameList>,
  supported_groups: &mut Option<SupportedGroups>,
) -> crate::Result<()> {
  match extension_ty {
    ExtensionTy::ApplicationLayerProtocolNegotiation => {
      duplicated_error(alpn.is_some())?;
      *alpn = Some(Alpn::decode(dw)?);
    }
    ExtensionTy::MaxFragmentLength => {
      duplicated_error(max_fragment_length.is_some())?;
      *max_fragment_length = Some(MaxFragmentLength::decode(dw)?);
    }
    ExtensionTy::ServerName => {
      duplicated_error(server_name.is_some())?;
      *server_name = Some(ServerNameList::decode(dw)?);
    }
    ExtensionTy::SupportedGroups => {
      duplicated_error(supported_groups.is_some())?;
      *supported_groups = Some(SupportedGroups::decode(dw)?);
    }
    ExtensionTy::ClientCertificateType
    | ExtensionTy::EarlyData
    | ExtensionTy::Heartbeat
    | ExtensionTy::ServerCertificateType
    | ExtensionTy::UseSrtp => {}
    ExtensionTy::CertificateAuthorities
    | ExtensionTy::Cookie
    | ExtensionTy::KeyShare
    | ExtensionTy::OidFilters
    | ExtensionTy::Padding
    | ExtensionTy::PostHandshakeAuth
    | ExtensionTy::PreSharedKey
    | ExtensionTy::PskKeyExchangeModes
    | ExtensionTy::SignatureAlgorithms
    | ExtensionTy::SignatureAlgorithmsCert
    | ExtensionTy::SignedCertificateTimestamp
    | ExtensionTy::StatusRequest
    | ExtensionTy::SupportedVersions => {
      return Err(TlsError::MismatchedExtension.into());
    }
  }
  Ok(())
}
