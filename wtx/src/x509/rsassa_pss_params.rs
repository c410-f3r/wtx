use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, ExplicitOpt, Len, SEQUENCE_TAG, U32, asn1_writer,
    decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
  x509::{
    AlgorithmIdentifier, EXPLICIT_TAG0, EXPLICIT_TAG1, EXPLICIT_TAG2, EXPLICIT_TAG3, X509Error,
  },
};

/// RSA metadata
#[derive(Debug, PartialEq)]
pub struct RsassaPssParams<B> {
  /// See [`AlgorithmIdentifier`].
  pub hash_algorithm: Option<AlgorithmIdentifier<B>>,
  /// See [`AlgorithmIdentifier`].
  pub mask_gen_algorithm: Option<AlgorithmIdentifier<B>>,
  /// Length of the salt value.
  pub salt_length: Option<u32>,
  /// Provides compatibility with IEEE Std 1363a-2004.
  pub trailer_field: Option<u32>,
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for RsassaPssParams<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidRsassaPssParams.into());
    };
    dw.bytes = value;
    let hash_algorithm = ExplicitOpt::<_, EXPLICIT_TAG0>::decode(dw)?.0;
    let mask_gen_algorithm = ExplicitOpt::<_, EXPLICIT_TAG1>::decode(dw)?.0;
    let salt_length: Option<U32> = ExplicitOpt::<_, EXPLICIT_TAG2>::decode(dw)?.0;
    let trailer_field: Option<U32> = ExplicitOpt::<_, EXPLICIT_TAG3>::decode(dw)?.0;
    dw.bytes = rest;
    Ok(Self {
      hash_algorithm,
      mask_gen_algorithm,
      salt_length: salt_length.map(|el| el.u32()),
      trailer_field: trailer_field.map(|el| el.u32()),
    })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for RsassaPssParams<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG, |local_ew| {
      ExplicitOpt::<_, EXPLICIT_TAG0>(self.hash_algorithm.as_ref()).encode(local_ew)?;
      ExplicitOpt::<_, EXPLICIT_TAG1>(self.mask_gen_algorithm.as_ref()).encode(local_ew)?;
      ExplicitOpt::<_, EXPLICIT_TAG2>(self.salt_length.map(U32::from_u32)).encode(local_ew)?;
      ExplicitOpt::<_, EXPLICIT_TAG3>(self.trailer_field.map(U32::from_u32)).encode(local_ew)?;
      Ok(())
    })
  }
}
