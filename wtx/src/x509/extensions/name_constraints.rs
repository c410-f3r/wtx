use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, Len, Opt, SEQUENCE_TAG, asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  collection::Vector,
  x509::{EXCLUDED_SUBTREES_TAG, GeneralSubtree, PERMITTED_SUBTREES_TAG, X509Error},
};

/// Indicates a name space within which all subject names in subsequent certificates in a
/// certification path MUST be located.
#[derive(Debug, PartialEq)]
pub struct NameConstraints<'bytes> {
  /// A set of permitted name subtrees.
  pub permitted_subtrees: Option<Vector<GeneralSubtree<'bytes>>>,
  /// A set of excluded name subtrees.
  pub excluded_subtrees: Option<Vector<GeneralSubtree<'bytes>>>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for NameConstraints<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidExtensionNameConstraints.into());
    };
    dw.bytes = value;
    let permitted_subtrees = Opt::decode_seq(dw, PERMITTED_SUBTREES_TAG)?.0;
    let excluded_subtrees = Opt::decode_seq(dw, EXCLUDED_SUBTREES_TAG)?.0;
    dw.bytes = rest;
    Ok(Self { permitted_subtrees, excluded_subtrees })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for NameConstraints<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG, |local_ew| {
      Opt(&self.permitted_subtrees).encode_seq(
        local_ew,
        Len::MAX_TWO_BYTES,
        PERMITTED_SUBTREES_TAG,
      )?;
      Opt(&self.excluded_subtrees).encode_seq(
        local_ew,
        Len::MAX_TWO_BYTES,
        EXCLUDED_SUBTREES_TAG,
      )?;
      Ok(())
    })
  }
}
