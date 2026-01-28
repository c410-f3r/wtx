// https://datatracker.ietf.org/doc/html/rfc7250#section-3

use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  tls::{
    TlsError, de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper,
    protocol::signature_scheme::SignatureScheme,
  },
};

const ELLIPTIC_CURVE_OID: &[u8] = &[0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01];
const ED25519_OID: &[u8] = &[0x06, 0x03, 0x2b, 0x65, 0x70];
const RSA_OID: &[u8] =
  &[0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01, 0x05, 0x00];
const SECP256R1_OID: &[u8] = &[
  0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, 0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d,
  0x03, 0x01, 0x07,
];
const SECP384R1_OID: &[u8] =
  &[0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, 0x06, 0x05, 0x2b, 0x81, 0x04, 0x00, 0x22];

pub(crate) struct RawPublicKey<'any> {
  algorithm: SignatureScheme,
  subject_public_key: &'any [u8],
}

impl<'de> Decode<'de, De> for RawPublicKey<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let (48, _, outer_content, rest) = decode_asn1_tlv(dw.bytes())? else {
      return Err(TlsError::InvalidRawPublicKey.into());
    };
    let (48, _, algorithm_bytes, after_algo) = decode_asn1_tlv(outer_content)? else {
      return Err(TlsError::InvalidRawPublicKey.into());
    };
    let (3, _, [0, subject_public_key @ ..], &[]) = decode_asn1_tlv(after_algo)? else {
      return Err(TlsError::InvalidRawPublicKey.into());
    };
    let algorithm = match algorithm_bytes {
      ED25519_OID => SignatureScheme::Ed25519,
      RSA_OID => {
        if dw.signature_scheme().is_rsa() {
          dw.signature_scheme()
        } else {
          return Err(TlsError::InvalidSignatureScheme.into());
        }
      }
      SECP256R1_OID => SignatureScheme::EcdsaSecp256r1Sha256,
      SECP384R1_OID => SignatureScheme::EcdsaSecp384r1Sha384,
      _ => return Err(TlsError::InvalidSignatureScheme.into()),
    };
    *dw.bytes_mut() = rest;
    Ok(Self { algorithm, subject_public_key })
  }
}

impl<'any> Encode<De> for RawPublicKey<'any> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    let algorithm_bytes = match self.algorithm {
      SignatureScheme::EcdsaSecp256r1Sha256 => SECP256R1_OID,
      SignatureScheme::EcdsaSecp384r1Sha384 => SECP384R1_OID,
      SignatureScheme::Ed25519 => ED25519_OID,
      SignatureScheme::RsaPkcs1Sha256
      | SignatureScheme::RsaPkcs1Sha384
      | SignatureScheme::RsaPssRsaeSha256
      | SignatureScheme::RsaPssRsaeSha384 => RSA_OID,
    };
    let algorithm_len = asn1_len(algorithm_bytes.len())?;
    let subject_public_key_len = asn1_len(self.subject_public_key.len().wrapping_add(1))?;
    let outer_len = 1usize
      .wrapping_add(algorithm_len.len().into())
      .wrapping_add(algorithm_bytes.len())
      .wrapping_add(1)
      .wrapping_add(subject_public_key_len.len().into())
      .wrapping_add(1)
      .wrapping_add(self.subject_public_key.len());
    ew.buffer().extend_from_slices([
      &[48][..],
      asn1_len(outer_len)?.as_slice(),
      &[48][..],
      &algorithm_len,
      algorithm_bytes,
      &[3][..],
      &subject_public_key_len,
      &[0][..],
      self.subject_public_key,
    ])?;
    Ok(())
  }
}

fn asn1_len(bytes_len: usize) -> crate::Result<ArrayVectorU8<u8, 3>> {
  let mut rslt = ArrayVectorU8::new();
  if let Ok(len) = u8::try_from(bytes_len) {
    if len <= 127 {
      rslt.push(len)?;
    } else {
      rslt.extend_from_copyable_slice(&[129, len])?;
    }
  } else if let Ok(len) = u16::try_from(bytes_len) {
    let [a, b] = len.to_be_bytes();
    rslt.extend_from_copyable_slice(&[130, a, b])?;
  } else {
    return Err(TlsError::InvalidAsn1Len.into());
  }
  Ok(rslt)
}

fn decode_asn1_tlv(bytes: &[u8]) -> crate::Result<(u8, u16, &[u8], &[u8])> {
  let [tag, maybe_len, maybe_after_len @ ..] = bytes else {
    return Err(TlsError::InvalidAsn1Tlv.into());
  };
  let (len, after_len) = if *maybe_len <= 127 {
    ((*maybe_len).into(), maybe_after_len)
  } else if *maybe_len == 129 {
    let [a, value @ ..] = maybe_after_len else {
      return Err(TlsError::InvalidAsn1Tlv.into());
    };
    ((*a).into(), value)
  } else if *maybe_len == 130 {
    let [a, b, value @ ..] = maybe_after_len else {
      return Err(TlsError::InvalidAsn1Tlv.into());
    };
    (u16::from_be_bytes([*a, *b]), value)
  } else {
    return Err(TlsError::InvalidAsn1Tlv.into());
  };
  let Some((value, rest)) = after_len.split_at_checked(len.into()) else {
    return Err(TlsError::InvalidAsn1Tlv.into());
  };
  Ok((*tag, len, value, rest))
}
