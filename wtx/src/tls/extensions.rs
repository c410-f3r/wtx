use crate::{
  de::{Decode, Encode},
  misc::{
    SuffixWriterMut,
    counter_writer::{CounterWriter, U16Counter},
  },
  tls::{
    TlsError, de::De, extension_ty::ExtensionTy, misc::u16_chunk,
    structs::server_name_list::ServerNameList,
  },
};

pub(crate) enum CertificateExtension {
  //StatusRequest(Unimplemented<'any>),
  //SignedCertificateTimestamp(Unimplemented<'any>),
}

pub(crate) enum CertificateRequestExtension {
  //StatusRequest(Unimplemented<'any>),
  //SignatureAlgorithms(SignatureAlgorithms<19>),
  //SignedCertificateTimestamp(Unimplemented<'any>),
  //CertificateAuthorities(Unimplemented<'any>),
  //OidFilters(Unimplemented<'any>),
  //SignatureAlgorithmsCert(Unimplemented<'any>),
}

pub(crate) enum ClientHelloExtension<'any> {
  ServerNameList(ServerNameList<'any>),
  //SupportedVersions(SupportedVersionsClientHello<16>),
  //SignatureAlgorithms(SignatureAlgorithms<19>),
  //SupportedGroups(SupportedGroups<16>),
  //KeyShare(KeyShareClientHello<'any, 1>),
  //PreSharedKey(PreSharedKeyClientHello<'any, 4>),
  //PskKeyExchangeModes(PskKeyExchangeModes<4>),
  //SignatureAlgorithmsCert(SignatureAlgorithmsCert<19>),
  //MaxFragmentLength(MaxFragmentLength),
  //StatusRequest(Unimplemented<'any>),
  //UseSrtp(Unimplemented<'any>),
  //Heartbeat(Unimplemented<'any>),
  //ApplicationLayerProtocolNegotiation(Unimplemented<'any>),
  //SignedCertificateTimestamp(Unimplemented<'any>),
  //ClientCertificateType(Unimplemented<'any>),
  //ServerCertificateType(Unimplemented<'any>),
  //Padding(Unimplemented<'any>),
  //EarlyData(Unimplemented<'any>),
  //Cookie(Unimplemented<'any>),
  //CertificateAuthorities(Unimplemented<'any>),
  //OidFilters(Unimplemented<'any>),
  //PostHandshakeAuth(Unimplemented<'any>),
}

impl<'de> Decode<'de, De> for ClientHelloExtension<'de> {
  #[inline]
  fn decode(aux: &mut (), dw: &mut &'de [u8]) -> crate::Result<Self> {
    let extension_ty = ExtensionTy::decode(aux, dw)?;
    u16_chunk(dw, TlsError::InvalidClientHello, |chunk| {
      Ok(match extension_ty {
        ExtensionTy::ServerName => Self::ServerNameList(ServerNameList::decode(aux, chunk)?),
        _ => {
          return Err(TlsError::InvalidClientHello.into());
        }
      })
    })
  }
}

impl<'de> Encode<De> for ClientHelloExtension<'de> {
  #[inline]
  fn encode(&self, aux: &mut (), sw: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    #[inline]
    fn cb<E>(
      aux: &mut (),
      encode: &E,
      extension_ty: ExtensionTy,
      sw: &mut SuffixWriterMut<'_>,
    ) -> crate::Result<()>
    where
      E: Encode<De>,
    {
      extension_ty.encode(aux, sw)?;
      U16Counter::default().write(sw, false, None, |local_ew| {
        encode.encode(aux, local_ew)?;
        Ok(())
      })
    }
    match self {
      Self::ServerNameList(elem) => cb(aux, elem, ExtensionTy::ServerName, sw),
    }
  }
}

pub(crate) enum EncryptedExtensionsExtension {
  //ServerName(ServerNameResponse),
  //MaxFragmentLength(MaxFragmentLength),
  //SupportedGroups(SupportedGroups<10>),
  //UseSrtp(Unimplemented<'any>),
  //Heartbeat(Unimplemented<'any>),
  //ApplicationLayerProtocolNegotiation(Unimplemented<'any>),
  //ClientCertificateType(Unimplemented<'any>),
  //ServerCertificateType(Unimplemented<'any>),
  //EarlyData(Unimplemented<'any>),
}

pub(crate) enum HelloRetryRequestExtension {
  //KeyShare(Unimplemented<'any>),
  //Cookie(Unimplemented<'any>),
  //SupportedVersions(Unimplemented<'any>),
}

pub(crate) enum NewSessionTicketExtension {
  //EarlyData(Unimplemented<'any>),
}

pub(crate) enum ServerHelloExtension {
  //KeyShare(KeyShareServerHello<'any>),
  //PreSharedKey(PreSharedKeyServerHello),
  //Cookie(Unimplemented<'any>), // temporary so we don't trip up on HelloRetryRequests
  //SupportedVersions(SupportedVersionsServerHello),
}
