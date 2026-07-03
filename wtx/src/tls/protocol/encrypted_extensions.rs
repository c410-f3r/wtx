// https://datatracker.ietf.org/doc/html/rfc8446#section-4.3.1

use crate::{
  codec::{Decode, Encode},
  misc::Lease,
  tls::{
    MaxFragmentLength, TlsCertificateTy, TlsError,
    de::De,
    misc::{duplicated_error, u16_chunk},
    protocol::{
      alpn::Alpn, cert_type::CertType, extension::Extension, extension_ty::ExtensionTy,
      server_name_list::ServerNameList, supported_groups::SupportedGroups,
    },
    tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Debug)]
pub(crate) struct EncryptedExtensions<B> {
  alpn: Option<Alpn>,
  client_cert_type: Option<TlsCertificateTy>,
  max_fragment_length: Option<MaxFragmentLength>,
  server_cert_type: Option<TlsCertificateTy>,
  server_name: Option<ServerNameList<B>>,
  supported_groups: Option<SupportedGroups>,
}

impl<B> EncryptedExtensions<B> {
  pub(crate) fn new(
    alpn: Option<Alpn>,
    client_cert_type: Option<TlsCertificateTy>,
    max_fragment_length: Option<MaxFragmentLength>,
    server_cert_type: Option<TlsCertificateTy>,
    server_name: Option<ServerNameList<B>>,
    supported_groups: Option<SupportedGroups>,
  ) -> Self {
    Self {
      alpn,
      client_cert_type,
      max_fragment_length,
      server_cert_type,
      server_name,
      supported_groups,
    }
  }

  pub(crate) const fn max_fragment_length(&self) -> Option<MaxFragmentLength> {
    self.max_fragment_length
  }
}

impl<'de, B> Decode<'de, De> for EncryptedExtensions<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let err = TlsError::InvalidEncryptedExtensions;
    let mut alpn = None;
    let mut client_cert_type = None;
    let mut max_fragment_length: Option<MaxFragmentLength> = None;
    let mut server_cert_type = None;
    let mut server_name = None;
    let mut supported_groups = None;
    u16_chunk(dw, err, |local_dw| {
      while !local_dw.bytes().is_empty() {
        let extension_ty = ExtensionTy::decode(local_dw)?;
        u16_chunk(local_dw, err, |local_local_dw| {
          manage_extension(
            &mut alpn,
            &mut client_cert_type,
            local_local_dw,
            extension_ty,
            &mut max_fragment_length,
            &mut server_cert_type,
            &mut server_name,
            &mut supported_groups,
          )
        })?;
      }
      Ok(())
    })?;
    Ok(Self {
      alpn,
      client_cert_type,
      max_fragment_length,
      server_cert_type,
      server_name,
      supported_groups,
    })
  }
}

impl<B> Encode<De> for EncryptedExtensions<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    let Self {
      alpn,
      client_cert_type,
      max_fragment_length,
      server_cert_type,
      server_name,
      supported_groups,
    } = self;
    Extension::new(ExtensionTy::ApplicationLayerProtocolNegotiation, alpn).encode(ew)?;
    if let Some(el) = client_cert_type {
      Extension::new(ExtensionTy::ClientCertificateType, CertType(*el)).encode(ew)?;
    }
    if let Some(el) = max_fragment_length {
      Extension::new(ExtensionTy::MaxFragmentLength, el).encode(ew)?;
    }
    if let Some(el) = server_cert_type {
      Extension::new(ExtensionTy::ServerCertificateType, CertType(*el)).encode(ew)?;
    }
    if let Some(el) = server_name {
      Extension::new(ExtensionTy::ServerName, el).encode(ew)?;
    }
    if let Some(el) = supported_groups {
      Extension::new(ExtensionTy::SupportedGroups, el).encode(ew)?;
    }
    Ok(())
  }
}

#[inline]
fn manage_extension<'de, B>(
  alpn: &mut Option<Alpn>,
  client_cert_type: &mut Option<TlsCertificateTy>,
  dw: &mut TlsDecodeWrapper<'de>,
  extension_ty: ExtensionTy,
  max_fragment_length: &mut Option<MaxFragmentLength>,
  server_cert_type: &mut Option<TlsCertificateTy>,
  server_name: &mut Option<ServerNameList<B>>,
  supported_groups: &mut Option<SupportedGroups>,
) -> crate::Result<()>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  match extension_ty {
    ExtensionTy::ApplicationLayerProtocolNegotiation => {
      duplicated_error(alpn.is_some())?;
      *alpn = Some(Alpn::decode(dw)?);
    }
    ExtensionTy::ClientCertificateType => {
      duplicated_error(client_cert_type.is_some())?;
      *client_cert_type = Some(CertType::decode(dw)?.0);
    }
    ExtensionTy::MaxFragmentLength => {
      duplicated_error(max_fragment_length.is_some())?;
      *max_fragment_length = Some(MaxFragmentLength::decode(dw)?);
    }
    ExtensionTy::ServerCertificateType => {
      duplicated_error(server_cert_type.is_some())?;
      *server_cert_type = Some(CertType::decode(dw)?.0);
    }
    ExtensionTy::ServerName => {
      duplicated_error(server_name.is_some())?;
      *server_name = Some(ServerNameList::<B>::decode(dw)?);
    }
    ExtensionTy::SupportedGroups => {
      duplicated_error(supported_groups.is_some())?;
      *supported_groups = Some(SupportedGroups::decode(dw)?);
    }
    ExtensionTy::EarlyData | ExtensionTy::Heartbeat | ExtensionTy::UseSrtp => {
      return Err(TlsError::UnsupportedExtension.into());
    }
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
