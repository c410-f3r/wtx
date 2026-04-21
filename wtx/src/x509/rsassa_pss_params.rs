use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, Len, Opt, SEQUENCE_TAG, U32, asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::{AlgorithmIdentifier, X509Error},
};

const TAG_HASH_ALGORITHM: u8 = 160;
const TAG_MASK_GEN_ALGORITHM: u8 = 161;
const TAG_SALT_LENGTH: u8 = 162;
const TAG_TRAILER_FIELD: u8 = 163;

/// RSA metadata
#[derive(Debug, PartialEq)]
pub struct RsassaPssParams<'bytes> {
  /// See [`AlgorithmIdentifier`].
  pub hash_algorithm: Option<AlgorithmIdentifier<'bytes>>,
  /// See [`AlgorithmIdentifier`].
  pub mask_gen_algorithm: Option<AlgorithmIdentifier<'bytes>>,
  /// Length of the salt value.
  pub salt_length: Option<u32>,
  /// Provides compatibility with IEEE Std 1363a-2004.
  pub trailer_field: Option<u32>,
}
impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for RsassaPssParams<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidRsassaPssParams.into());
    };
    dw.bytes = value;
    let hash_algorithm = Opt::decode(dw, TAG_HASH_ALGORITHM)?.0;
    let mask_gen_algorithm = Opt::decode(dw, TAG_MASK_GEN_ALGORITHM)?.0;
    let salt_length: Option<U32> = Opt::decode(dw, TAG_SALT_LENGTH)?.0;
    let trailer_field: Option<U32> = Opt::decode(dw, TAG_TRAILER_FIELD)?.0;
    dw.bytes = rest;
    Ok(Self {
      hash_algorithm,
      mask_gen_algorithm,
      salt_length: salt_length.map(|el| el.u32()),
      trailer_field: trailer_field.map(|el| el.u32()),
    })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for RsassaPssParams<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG, |local_ew| {
      Opt(&self.hash_algorithm).encode(local_ew, TAG_HASH_ALGORITHM)?;
      Opt(&self.mask_gen_algorithm).encode(local_ew, TAG_MASK_GEN_ALGORITHM)?;
      Opt(&self.salt_length.map(U32::from_u32)).encode(local_ew, TAG_SALT_LENGTH)?;
      Opt(&self.trailer_field.map(U32::from_u32)).encode(local_ew, TAG_TRAILER_FIELD)?;
      Ok(())
    })
  }
}
