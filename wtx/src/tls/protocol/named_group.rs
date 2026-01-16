use crate::{
  de::{Decode, Encode},
  misc::SuffixWriterMut,
  tls::de::De,
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
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    Ok(Self::try_from(<u16 as Decode<De>>::decode(dw)?)?)
  }
}

impl Encode<De> for NamedGroup {
  #[inline]
  fn encode(&self, ew: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    ew.extend_from_slice(&u16::from(*self).to_be_bytes())?;
    Ok(())
  }
}

#[derive(Debug)]
pub(crate) enum NamedGroupParam<A, B, C> {
  /// Secp256r1
  Secp256r1(A),
  /// Secp384r1
  Secp384r1(B),
  /// X25519
  X25519(C),
}

#[cfg(feature = "aws-lc-rs")]
pub(crate) mod aws_lc_rs {
  use crate::{
    collection::ArrayVectorU8,
    rng::CryptoRng,
    tls::{
      MAX_PK_LEN, NamedGroup,
      cipher_suite::{P256AwsLcRs, P384AwsLcRs, X25519AwsLcRs},
      protocol::{ephemeral_secret_key::EphemeralSecretKey, named_group::NamedGroupParam},
    },
  };
  use aws_lc_rs::rand::SystemRandom;

  pub(crate) type NamedGroupParamAwsLcRs = NamedGroupParam<
    aws_lc_rs::agreement::EphemeralPrivateKey,
    aws_lc_rs::agreement::EphemeralPrivateKey,
    aws_lc_rs::agreement::EphemeralPrivateKey,
  >;

  impl EphemeralSecretKey for NamedGroupParamAwsLcRs {
    type SharedSecret = ();

    fn random<RNG>(ng: NamedGroup, _rng: &mut RNG) -> crate::Result<Self>
    where
      RNG: CryptoRng,
    {
      let rng = SystemRandom::new();
      Ok(match ng {
        NamedGroup::Secp256r1 => {
          NamedGroupParam::Secp256r1(aws_lc_rs::agreement::EphemeralPrivateKey::generate(
            const { &P256AwsLcRs::new().0 },
            &rng,
          )?)
        }
        NamedGroup::Secp384r1 => {
          NamedGroupParam::Secp384r1(aws_lc_rs::agreement::EphemeralPrivateKey::generate(
            const { &P384AwsLcRs::new().0 },
            &rng,
          )?)
        }
        NamedGroup::X25519 => {
          NamedGroupParam::X25519(aws_lc_rs::agreement::EphemeralPrivateKey::generate(
            const { &X25519AwsLcRs::new().0 },
            &rng,
          )?)
        }
      })
    }

    fn diffie_hellman(&self, pk: &[u8]) -> Self::SharedSecret {}

    fn public_key(&self) -> crate::Result<ArrayVectorU8<u8, MAX_PK_LEN>> {
      let inner = match self {
        NamedGroupParam::Secp256r1(el) => el,
        NamedGroupParam::Secp384r1(el) => el,
        NamedGroupParam::X25519(el) => el,
      };
      let mut array = ArrayVectorU8::new();
      let _rslt = array.extend_from_copyable_slice(inner.compute_public_key()?.as_ref());
      Ok(array)
    }
  }
}

#[cfg(feature = "rust-crypto")]
pub(crate) mod rust_crypto {
  use crypto_common::Generate;

use crate::{
    collection::ArrayVectorU8,
    rng::CryptoRng,
    tls::{
      MAX_PK_LEN, NamedGroup,
      protocol::{ephemeral_secret_key::EphemeralSecretKey, named_group::NamedGroupParam},
    },
  };

  pub(crate) type NamedGroupParamRustCrypto = NamedGroupParam<
    p256::ecdh::EphemeralSecret,
    p384::ecdh::EphemeralSecret,
    x25519_dalek::EphemeralSecret,
  >;

  impl EphemeralSecretKey for NamedGroupParamRustCrypto {
    type SharedSecret = ();

    fn random<RNG>(ng: NamedGroup, rng: &mut RNG) -> crate::Result<Self>
    where
      RNG: CryptoRng,
    { 
      Ok(match ng {
        NamedGroup::Secp256r1 => NamedGroupParamRustCrypto::Secp256r1(
          match p256::ecdh::EphemeralSecret::try_generate_from_rng(rng) {
            Ok(el) => el,
          },
        ),
        NamedGroup::Secp384r1 => NamedGroupParamRustCrypto::Secp384r1(
          match p384::ecdh::EphemeralSecret::try_generate_from_rng(rng) {
            Ok(el) => el,
          },
        ),
        NamedGroup::X25519 => {
          NamedGroupParamRustCrypto::X25519(x25519_dalek::EphemeralSecret::random_from_rng(rng))
        }
      })
    }

    fn diffie_hellman(&self, pk: &[u8]) -> Self::SharedSecret {}

    fn public_key(&self) -> crate::Result<ArrayVectorU8<u8, MAX_PK_LEN>> {
      let mut array = ArrayVectorU8::new();
      match self {
        NamedGroupParam::Secp256r1(el) => {
          let _rslt =
            array.extend_from_copyable_slice(p256::EncodedPoint::from(el.public_key()).as_bytes());
        }
        NamedGroupParam::Secp384r1(el) => {
          let _rslt =
            array.extend_from_copyable_slice(p384::EncodedPoint::from(el.public_key()).as_bytes());
        }
        NamedGroupParam::X25519(el) => {
          let _rslt =
            array.extend_from_copyable_slice(x25519_dalek::PublicKey::from(el).as_bytes());
        }
      }
      Ok(array)
    }
  }
}
