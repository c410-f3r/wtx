use crate::{
  crypto::{Agreement, CryptoError, P256Openssl, P384Openssl, X25519Openssl},
  rng::CryptoRng,
};
use openssl::{
  bn::BigNumContext,
  derive::Deriver,
  ec::{EcGroup, EcKey, EcPoint, PointConversionForm},
  nid::Nid,
  pkey::{PKey, Private},
};

impl Agreement for P256Openssl {
  type EphemeralSecretKey = PKey<Private>;
  type PublicKey = [u8; 65];
  type SharedSecret = [u8; 32];

  #[inline]
  fn diffie_hellman(
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    local_diffie_hellman(Nid::X9_62_PRIME256V1, esk, other_participant_pk)
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(_: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
    Ok(PKey::from_ec_key(EcKey::generate(&group)?)?)
  }

  #[inline]
  fn public_key(esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    let ec_key = esk.ec_key()?;
    let mut ctx = BigNumContext::new()?;
    let mut bytes = [0u8; 65];
    bytes.copy_from_slice(&ec_key.public_key().to_bytes(
      ec_key.group(),
      PointConversionForm::UNCOMPRESSED,
      &mut ctx,
    )?);
    Ok(bytes)
  }
}

impl Agreement for P384Openssl {
  type EphemeralSecretKey = PKey<Private>;
  type PublicKey = [u8; 97];
  type SharedSecret = [u8; 48];

  #[inline]
  fn diffie_hellman(
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    local_diffie_hellman(Nid::SECP384R1, esk, other_participant_pk)
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(_: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    let group = EcGroup::from_curve_name(Nid::SECP384R1)?;
    Ok(PKey::from_ec_key(EcKey::generate(&group)?)?)
  }

  #[inline]
  fn public_key(esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    let ec_key = esk.ec_key()?;
    let mut ctx = BigNumContext::new()?;
    let mut bytes = [0u8; 97];
    bytes.copy_from_slice(&ec_key.public_key().to_bytes(
      ec_key.group(),
      PointConversionForm::UNCOMPRESSED,
      &mut ctx,
    )?);
    Ok(bytes)
  }
}

impl Agreement for X25519Openssl {
  type EphemeralSecretKey = PKey<Private>;
  type PublicKey = [u8; 32];
  type SharedSecret = [u8; 32];

  #[inline]
  fn diffie_hellman(
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    let peer = PKey::public_key_from_raw_bytes(other_participant_pk, esk.id())?;
    let mut deriver = Deriver::new(&esk)?;
    deriver.set_peer(&peer)?;
    let mut secret = [0; 32];
    let _ = deriver.derive(&mut secret)?;
    Ok(secret)
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(_: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    Ok(PKey::generate_x25519()?)
  }

  #[inline]
  fn public_key(esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    esk.raw_public_key()?.try_into().map_err(|_err| CryptoError::PublicKeyAgreementError.into())
  }
}

fn local_diffie_hellman<const N: usize>(
  nid: Nid,
  esk: PKey<Private>,
  other_participant_pk: &[u8],
) -> crate::Result<[u8; N]> {
  let group = EcGroup::from_curve_name(nid)?;
  let mut ctx = BigNumContext::new()?;
  let point = EcPoint::from_bytes(&group, other_participant_pk, &mut ctx)?;
  let peer = PKey::from_ec_key(EcKey::from_public_key(&group, &point)?)?;
  let mut deriver = Deriver::new(&esk)?;
  deriver.set_peer(&peer)?;
  let mut secret = [0; N];
  let _ = deriver.derive(&mut secret)?;
  Ok(secret)
}
