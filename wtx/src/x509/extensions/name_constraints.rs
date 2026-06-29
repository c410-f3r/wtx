use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Len, Opt, SEQUENCE_TAG, asn1_writer,
    decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collections::ArrayVectorU8,
  misc::Lease,
  x509::{EXCLUDED_SUBTREES_TAG, GeneralSubtree, PERMITTED_SUBTREES_TAG, X509Error},
};

/// Indicates a name space within which all subject names in subsequent certificates in a
/// certification path MUST be located.
#[derive(Clone, Debug, PartialEq)]
pub struct NameConstraints<B> {
  /// A set of permitted name subtrees.
  pub permitted_subtrees: Option<ArrayVectorU8<GeneralSubtree<B>, 2>>,
  /// A set of excluded name subtrees.
  pub excluded_subtrees: Option<ArrayVectorU8<GeneralSubtree<B>, 2>>,
}

impl<B> NameConstraints<B> {
  /// Shortcut
  #[inline]
  pub const fn new(
    permitted_subtrees: Option<ArrayVectorU8<GeneralSubtree<B>, 2>>,
    excluded_subtrees: Option<ArrayVectorU8<GeneralSubtree<B>, 2>>,
  ) -> Self {
    Self { permitted_subtrees, excluded_subtrees }
  }
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for NameConstraints<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidExtensionNameConstraints.into());
    };
    dw.bytes = value;
    let mut fun = || {
      let permitted_subtrees = Opt::decode_seq(dw, PERMITTED_SUBTREES_TAG).ok()?.0;
      let excluded_subtrees = Opt::decode_seq(dw, EXCLUDED_SUBTREES_TAG).ok()?.0;
      Some((permitted_subtrees, excluded_subtrees))
    };
    let (permitted_subtrees, excluded_subtrees) =
      fun().ok_or(X509Error::InvalidExtensionNameConstraints)?;
    dw.bytes = rest;
    Ok(Self { permitted_subtrees, excluded_subtrees })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for NameConstraints<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
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
