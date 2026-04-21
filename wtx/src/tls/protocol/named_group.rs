use crate::{
  crypto::Agreement,
  codec::{Decode, Encode},
  rng::CryptoRng,
  tls::{de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper},
};

create_enum! {
  /// Specifies the group or curve used for key exchange mechanisms.
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum NamedGroup<u16> {
    /// Secp256r1
    Secp256r1 = (23),
    /// Secp384r1
    Secp384r1 = (24),
    /// X25519
    X25519 = (29),
  }
}

impl<'de> Decode<'de, De> for NamedGroup {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    Ok(Self::try_from(<u16 as Decode<De>>::decode(dw)?)?)
  }
}

impl Encode<De> for NamedGroup {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_slice(&u16::from(*self).to_be_bytes())?;
    Ok(())
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

impl<A, B, C> Agreement for NamedGroupParam<A, B, C>
where
  A: Agreement,
  B: Agreement,
  C: Agreement,
{
  type EphemeralSecretKey =
    NamedGroupParam<A::EphemeralSecretKey, B::EphemeralSecretKey, C::EphemeralSecretKey>;
  type PublicKey = NamedGroupParam<A::PublicKey, B::PublicKey, C::PublicKey>;
  type SharedSecret = NamedGroupParam<A::SharedSecret, B::SharedSecret, C::SharedSecret>;

  #[inline]
  fn diffie_hellman(
    &self,
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    Ok(match self {
      Self::Secp256r1(el) => {
        let NamedGroupParam::Secp256r1(esk_inner) = esk else {
          panic!();
        };
        NamedGroupParam::Secp256r1(el.diffie_hellman(esk_inner, other_participant_pk)?)
      }
      Self::Secp384r1(el) => {
        let NamedGroupParam::Secp384r1(esk_inner) = esk else {
          panic!();
        };
        NamedGroupParam::Secp384r1(el.diffie_hellman(esk_inner, other_participant_pk)?)
      }
      Self::X25519(el) => {
        let NamedGroupParam::X25519(esk_inner) = esk else {
          panic!();
        };
        NamedGroupParam::X25519(el.diffie_hellman(esk_inner, other_participant_pk)?)
      }
    })
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(&self, rng: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    Ok(match self {
      Self::Secp256r1(el) => NamedGroupParam::Secp256r1(el.ephemeral_secret_key(rng)?),
      Self::Secp384r1(el) => NamedGroupParam::Secp384r1(el.ephemeral_secret_key(rng)?),
      Self::X25519(el) => NamedGroupParam::X25519(el.ephemeral_secret_key(rng)?),
    })
  }

  #[inline]
  fn public_key(&self, esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    Ok(match self {
      Self::Secp256r1(el) => {
        let NamedGroupParam::Secp256r1(esk_inner) = esk else {
          panic!();
        };
        NamedGroupParam::Secp256r1(el.public_key(esk_inner)?)
      }
      Self::Secp384r1(el) => {
        let NamedGroupParam::Secp384r1(esk_inner) = esk else {
          panic!();
        };
        NamedGroupParam::Secp384r1(el.public_key(esk_inner)?)
      }
      Self::X25519(el) => {
        let NamedGroupParam::X25519(esk_inner) = esk else {
          panic!();
        };
        NamedGroupParam::X25519(el.public_key(esk_inner)?)
      }
    })
  }
}

impl<A, B, C> AsRef<[u8]> for NamedGroupParam<A, B, C>
where
  A: AsRef<[u8]>,
  B: AsRef<[u8]>,
  C: AsRef<[u8]>,
{
  #[inline]
  fn as_ref(&self) -> &[u8] {
    match self {
      Self::Secp256r1(el) => el.as_ref(),
      Self::Secp384r1(el) => el.as_ref(),
      Self::X25519(el) => el.as_ref(),
    }
  }
}

impl<A, B, C> From<&NamedGroupParam<A, B, C>> for NamedGroup {
  #[inline]
  fn from(value: &NamedGroupParam<A, B, C>) -> Self {
    match value {
      NamedGroupParam::Secp256r1(_) => NamedGroup::Secp256r1,
      NamedGroupParam::Secp384r1(_) => NamedGroup::Secp384r1,
      NamedGroupParam::X25519(_) => NamedGroup::X25519,
    }
  }
}

impl<A, B, C> From<NamedGroup> for NamedGroupParam<A, B, C>
where
  A: Default,
  B: Default,
  C: Default,
{
  #[inline]
  fn from(value: NamedGroup) -> Self {
    match value {
      NamedGroup::Secp256r1 => NamedGroupParam::Secp256r1(A::default()),
      NamedGroup::Secp384r1 => NamedGroupParam::Secp384r1(B::default()),
      NamedGroup::X25519 => NamedGroupParam::X25519(C::default()),
    }
  }
}
