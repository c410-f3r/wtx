use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, Len, Opt, SEQUENCE_TAG, U32, asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::{INHIBIT_POLICY_MAPPING_TAG, REQUIRE_EXPLICIT_POLICY_TAG, X509Error},
};

/// It can be used to prohibit policy mapping or require that each certificate in a path contain
/// an acceptable policy identifier.
#[derive(Debug, PartialEq)]
pub struct PolicyConstraints {
  /// Number of additional certificates that may appear in the path before an explicit policy
  /// is required.
  pub require_explicit_policy: Option<U32>,
  /// Number of additional certificates that may appear in the path before policy mapping
  /// is no longer permitted.
  pub inhibit_policy_mapping: Option<U32>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for PolicyConstraints {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidExtensionPolicyConstraints.into());
    };
    dw.bytes = value;
    let require_explicit_policy = Opt::decode(dw, REQUIRE_EXPLICIT_POLICY_TAG)?.0;
    let inhibit_policy_mapping = Opt::decode(dw, INHIBIT_POLICY_MAPPING_TAG)?.0;
    dw.bytes = rest;
    Ok(Self { require_explicit_policy, inhibit_policy_mapping })
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for PolicyConstraints {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG, |local_ew| {
      Opt(&self.require_explicit_policy).encode(local_ew, REQUIRE_EXPLICIT_POLICY_TAG)?;
      Opt(&self.inhibit_policy_mapping).encode(local_ew, INHIBIT_POLICY_MAPPING_TAG)?;
      Ok(())
    })
  }
}
