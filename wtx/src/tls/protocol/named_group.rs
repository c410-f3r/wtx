use crate::{
  de::{Decode, Encode},
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

#[derive(Debug)]
pub(crate) enum NamedGroupParam<A, B, C> {
  /// Secp256r1
  Secp256r1(A),
  /// Secp384r1
  Secp384r1(B),
  /// X25519
  X25519(C),
}

impl<A, B, C> NamedGroupParam<A, B, C> {
  pub(crate) fn simplify(&self) -> NamedGroup {
    match self {
      NamedGroupParam::Secp256r1(_) => NamedGroup::Secp256r1,
      NamedGroupParam::Secp384r1(_) => NamedGroup::Secp384r1,
      NamedGroupParam::X25519(_) => NamedGroup::X25519,
    }
  }
}

#[cfg(feature = "aws-lc-rs")]
pub(crate) mod aws_lc_rs {
  use crate::{
    collection::ArrayVectorU8,
    rng::CryptoRng,
    tls::{
      MAX_PK_LEN, NamedGroup, TlsError,
      ephemeral_secret_key::EphemeralSecretKey,
      protocol::{
        cipher_suite_wrappers::{P256AwsLcRs, P384AwsLcRs, X25519AwsLcRs},
        named_group::NamedGroupParam,
      },
    },
  };
  use aws_lc_rs::rand::SystemRandom;

  pub(crate) type NamedGroupEpkAwsLcRs = NamedGroupParam<
    aws_lc_rs::agreement::EphemeralPrivateKey,
    aws_lc_rs::agreement::EphemeralPrivateKey,
    aws_lc_rs::agreement::EphemeralPrivateKey,
  >;
  pub(crate) type NamedGroupScAwsLcRs =
    NamedGroupParam<ArrayVectorU8<u8, 32>, ArrayVectorU8<u8, 32>, ArrayVectorU8<u8, 32>>;

  impl EphemeralSecretKey for NamedGroupEpkAwsLcRs {
    type SharedSecret = ArrayVectorU8<u8, 32>;

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

    fn diffie_hellman(self, pk: &[u8]) -> crate::Result<Self::SharedSecret> {
      let (algorithm, epk) = match self {
        NamedGroupParam::Secp256r1(el) => (&aws_lc_rs::agreement::ECDH_P256, el),
        NamedGroupParam::Secp384r1(el) => (&aws_lc_rs::agreement::ECDH_P384, el),
        NamedGroupParam::X25519(el) => (&aws_lc_rs::agreement::X25519, el),
      };
      let mut secret = ArrayVectorU8::new();
      aws_lc_rs::agreement::agree_ephemeral(
        epk,
        aws_lc_rs::agreement::UnparsedPublicKey::new(algorithm, pk),
        TlsError::DiffieHellmanError.into(),
        |value| {
          secret.extend_from_copyable_slice(value)?;
          crate::Result::Ok(())
        },
      )?;
      Ok(secret)
    }

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
      MAX_PK_LEN, NamedGroup, TlsError, ephemeral_secret_key::EphemeralSecretKey,
      protocol::named_group::NamedGroupParam, shared_secret::SharedSecret,
    },
  };

  pub(crate) type NamedGroupEpkRustCrypto = NamedGroupParam<
    p256::ecdh::EphemeralSecret,
    p384::ecdh::EphemeralSecret,
    x25519_dalek::EphemeralSecret,
  >;
  pub(crate) type NamedGroupScRustCrypto =
    NamedGroupParam<p256::ecdh::SharedSecret, p384::ecdh::SharedSecret, x25519_dalek::SharedSecret>;

  impl EphemeralSecretKey for NamedGroupEpkRustCrypto {
    type SharedSecret = NamedGroupScRustCrypto;

    fn random<RNG>(ng: NamedGroup, rng: &mut RNG) -> crate::Result<Self>
    where
      RNG: CryptoRng,
    {
      Ok(match ng {
        NamedGroup::Secp256r1 => NamedGroupEpkRustCrypto::Secp256r1(
          match p256::ecdh::EphemeralSecret::try_generate_from_rng(rng) {
            Ok(el) => el,
          },
        ),
        NamedGroup::Secp384r1 => NamedGroupEpkRustCrypto::Secp384r1(
          match p384::ecdh::EphemeralSecret::try_generate_from_rng(rng) {
            Ok(el) => el,
          },
        ),
        NamedGroup::X25519 => {
          NamedGroupEpkRustCrypto::X25519(x25519_dalek::EphemeralSecret::random_from_rng(rng))
        }
      })
    }

    fn diffie_hellman(self, pk: &[u8]) -> crate::Result<Self::SharedSecret> {
      match self {
        NamedGroupParam::Secp256r1(el) => Ok(NamedGroupParam::Secp256r1(el.diffie_hellman(
          &p256::PublicKey::from_sec1_bytes(pk).map_err(|_err| TlsError::DiffieHellmanError)?,
        ))),
        NamedGroupParam::Secp384r1(el) => Ok(NamedGroupParam::Secp384r1(el.diffie_hellman(
          &p384::PublicKey::from_sec1_bytes(pk).map_err(|_err| TlsError::DiffieHellmanError)?,
        ))),
        NamedGroupParam::X25519(el) => {
          let array: [u8; 32] = pk.try_into()?;
          Ok(NamedGroupParam::X25519(el.diffie_hellman(&x25519_dalek::PublicKey::from(array))))
        }
      }
    }

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

  impl SharedSecret for NamedGroupScRustCrypto {}
}
