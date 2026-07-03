use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorU8,
  misc::{
    Lease,
    counter_writer::{CounterWriterBytesTy, u8_write},
  },
  tls::{
    TlsError,
    de::De,
    misc::{u8_chunk, u16_chunk},
    protocol::extension_ty::ExtensionTy,
    tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Clone, Debug)]
/// PSK for future handshakes
pub struct NewSessionTicket<B> {
  ticket_lifetime: u32,
  ticket_age_add: u32,
  ticket_nonce: ArrayVectorU8<u8, 32>,
  opaque: B,
}

impl<B> NewSessionTicket<B> {
  #[inline]
  /// The actual encrypted data.
  pub const fn opaque(&self) -> &B {
    &self.opaque
  }

  #[inline]
  /// A random value used by the client to obscure the ticket's age.
  pub const fn ticket_age_add(&self) -> u32 {
    self.ticket_age_add
  }

  #[inline]
  /// Indicates the duration (in seconds) that the ticket is valid.
  pub const fn ticket_lifetime(&self) -> u32 {
    self.ticket_lifetime
  }

  #[inline]
  /// A unique value used to differentiate multiple tickets.
  pub const fn ticket_nonce(&self) -> &ArrayVectorU8<u8, 32> {
    &self.ticket_nonce
  }
}

impl<'de, B> Decode<'de, De> for NewSessionTicket<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let err = TlsError::InvalidNewSessionTicket;
    let ticket_lifetime: u32 = Decode::<'_, De>::decode(dw)?;
    let ticket_age_add: u32 = Decode::<'_, De>::decode(dw)?;
    let ticket_nonce = u8_chunk(dw, err, |el| Ok(el.bytes()))?.try_into()?;
    let len: u16 = Decode::<'_, De>::decode(dw)?;
    let Some((opaque, rest)) = dw.bytes().split_at_checked(len.into()) else {
      return Err(TlsError::InvalidServerName.into());
    };
    *dw.bytes_mut() = rest;
    u16_chunk(dw, err, |local_dw| {
      while !local_dw.bytes().is_empty() {
        let extension_ty = ExtensionTy::decode(local_dw)?;
        u16_chunk(local_dw, err, |local_local_dw| manage_extension(local_local_dw, extension_ty))?;
      }
      Ok(())
    })?;
    Ok(Self {
      ticket_lifetime,
      ticket_age_add,
      ticket_nonce,
      opaque: opaque.try_into().map_err(Into::into)?,
    })
  }
}

impl<B> Encode<De> for NewSessionTicket<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    <u32 as Encode<De>>::encode(&self.ticket_lifetime, ew)?;
    <u32 as Encode<De>>::encode(&self.ticket_age_add, ew)?;
    u8_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      local_ew.buffer().extend_from_copyable_slice(self.ticket_nonce.lease())?;
      crate::Result::Ok(())
    })?;
    u8_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      local_ew.buffer().extend_from_copyable_slice(self.opaque.lease())?;
      crate::Result::Ok(())
    })?;
    Ok(())
  }
}

#[inline]
fn manage_extension(
  _dw: &mut TlsDecodeWrapper<'_>,
  extension_ty: ExtensionTy,
) -> crate::Result<()> {
  match extension_ty {
    ExtensionTy::EarlyData => Err(TlsError::UnsupportedExtension.into()),
    ExtensionTy::ApplicationLayerProtocolNegotiation
    | ExtensionTy::CertificateAuthorities
    | ExtensionTy::ClientCertificateType
    | ExtensionTy::Cookie
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
    | ExtensionTy::SignedCertificateTimestamp
    | ExtensionTy::StatusRequest
    | ExtensionTy::SupportedGroups
    | ExtensionTy::SupportedVersions
    | ExtensionTy::UseSrtp => Err(TlsError::MismatchedExtension.into()),
  }
}
