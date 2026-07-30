use crate::{
  codec::{Decode, Encode},
  crypto::{Agreement, EcdhP256Global, EcdhP384Global, X25519Global},
  misc::Lease,
  rng::CryptoRng,
  tls::{
    AlertDescription, TlsError, de::De, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

pub(crate) type NamedGroupAgreement = NamedGroupParam<EcdhP256Global, EcdhP384Global, X25519Global>;
pub(crate) type NamedGroupPk = NamedGroupParam<
  <EcdhP256Global as Agreement>::PublicKey,
  <EcdhP384Global as Agreement>::PublicKey,
  <X25519Global as Agreement>::PublicKey,
>;
pub(crate) type NamedGroupSs = NamedGroupParam<
  <EcdhP256Global as Agreement>::SharedSecret,
  <EcdhP384Global as Agreement>::SharedSecret,
  <X25519Global as Agreement>::SharedSecret,
>;

/// Specifies the group or curve used for key exchange mechanisms.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NamedGroup {
  /// Secp256r1
  Secp256r1 = 23,
  /// Secp384r1
  Secp384r1 = 24,
  /// X25519
  #[default]
  X25519 = 29,
}

impl NamedGroup {
  pub(crate) const PRIORITY: [Self; 3] = [Self::X25519, Self::Secp256r1, Self::Secp384r1];

  pub(crate) fn agreement<RNG>(self, rng: &mut RNG) -> crate::Result<NamedGroupAgreement>
  where
    RNG: CryptoRng,
  {
    Ok(match self {
      NamedGroup::Secp256r1 => NamedGroupAgreement::Secp256r1(EcdhP256Global::generate(rng)?),
      NamedGroup::Secp384r1 => NamedGroupAgreement::Secp384r1(EcdhP384Global::generate(rng)?),
      NamedGroup::X25519 => NamedGroupAgreement::X25519(X25519Global::generate(rng)?),
    })
  }

  pub(crate) const fn len() -> usize {
    3
  }
}

impl<'de> Decode<'de, De> for NamedGroup {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    Self::try_from(<u16 as Decode<De>>::decode(dw)?)
  }
}

impl Encode<De> for NamedGroup {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_copyable_slice(&u16::from(*self).to_be_bytes())?;
    Ok(())
  }
}

impl From<NamedGroup> for u16 {
  #[inline]
  fn from(value: NamedGroup) -> Self {
    match value {
      NamedGroup::Secp256r1 => 23,
      NamedGroup::Secp384r1 => 24,
      NamedGroup::X25519 => 29,
    }
  }
}

impl TryFrom<u16> for NamedGroup {
  type Error = crate::Error;
  #[inline]
  fn try_from(value: u16) -> crate::Result<Self> {
    Ok(match value {
      23 => NamedGroup::Secp256r1,
      24 => NamedGroup::Secp384r1,
      29 => NamedGroup::X25519,
      _ => return Err(TlsError::UnknownNamedGroup.into()),
    })
  }
}

impl NamedGroupAgreement {
  pub(crate) fn diffie_hellman<const IS_CLIENT: bool>(
    self,
    other_participant_pk: &[u8],
  ) -> crate::Result<NamedGroupSs> {
    let description =
      if IS_CLIENT { AlertDescription::BadRecordMac } else { AlertDescription::IllegalParameter };
    Ok(match self {
      NamedGroupParam::Secp256r1(elem) => {
        NamedGroupSs::Secp256r1(match elem.diffie_hellman(other_participant_pk) {
          Ok(el) => el,
          Err(_err) => {
            return Err(crate::Error::TlsErrorReply(TlsError::DiffieHellmanError, description));
          }
        })
      }
      NamedGroupParam::Secp384r1(elem) => {
        NamedGroupSs::Secp384r1(match elem.diffie_hellman(other_participant_pk) {
          Ok(el) => el,
          Err(_err) => {
            return Err(crate::Error::TlsErrorReply(TlsError::DiffieHellmanError, description));
          }
        })
      }
      NamedGroupParam::X25519(elem) => {
        NamedGroupSs::X25519(match elem.diffie_hellman(other_participant_pk) {
          Ok(el) => el,
          Err(_err) => {
            return Err(crate::Error::TlsErrorReply(TlsError::DiffieHellmanError, description));
          }
        })
      }
    })
  }

  pub(crate) fn public_key(&self) -> crate::Result<NamedGroupPk> {
    Ok(match self {
      NamedGroupParam::Secp256r1(elem) => NamedGroupPk::Secp256r1(elem.public_key()?),
      NamedGroupParam::Secp384r1(elem) => NamedGroupPk::Secp384r1(elem.public_key()?),
      NamedGroupParam::X25519(elem) => NamedGroupPk::X25519(elem.public_key()?),
    })
  }
}

/// A version of [`NamedGroup`] with associated parameters.
#[derive(Debug)]
pub enum NamedGroupParam<A, B, C> {
  /// Secp256r1
  Secp256r1(A),
  /// Secp384r1
  Secp384r1(B),
  /// X25519
  X25519(C),
}

impl<A, B, C> NamedGroupParam<A, B, C> {
  #[inline]
  pub(crate) fn named_group(&self) -> NamedGroup {
    match self {
      NamedGroupParam::Secp256r1(_) => NamedGroup::Secp256r1,
      NamedGroupParam::Secp384r1(_) => NamedGroup::Secp384r1,
      NamedGroupParam::X25519(_) => NamedGroup::X25519,
    }
  }
}

impl<A, B, C, T> AsRef<[T]> for NamedGroupParam<A, B, C>
where
  A: AsRef<[T]>,
  B: AsRef<[T]>,
  C: AsRef<[T]>,
{
  #[inline]
  fn as_ref(&self) -> &[T] {
    match self {
      NamedGroupParam::Secp256r1(el) => el.as_ref(),
      NamedGroupParam::Secp384r1(el) => el.as_ref(),
      NamedGroupParam::X25519(el) => el.as_ref(),
    }
  }
}

// `AsRef` because of third parties
impl<A, B, C, T> Lease<[T]> for NamedGroupParam<A, B, C>
where
  A: AsRef<[T]>,
  B: AsRef<[T]>,
  C: AsRef<[T]>,
{
  #[inline]
  fn lease(&self) -> &[T] {
    match self {
      NamedGroupParam::Secp256r1(el) => el.as_ref(),
      NamedGroupParam::Secp384r1(el) => el.as_ref(),
      NamedGroupParam::X25519(el) => el.as_ref(),
    }
  }
}
