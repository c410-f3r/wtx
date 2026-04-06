use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, Boolean, Len, Opt, SEQUENCE_TAG, asn1_writer,
    decode_asn1_tlv,
  },
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::{
    DISTRIBUTION_POINT_TAG, DistributionPointName, INDIRECT_CRL_TAG,
    ONLY_CONTAINS_ATTRIBUTE_CERTS_TAG, ONLY_CONTAINS_CA_CERTS_TAG, ONLY_CONTAINS_USER_CERTS_TAG,
    ONLY_SOME_REASONS_TAG, ReasonFlags, X509Error,
  },
};

/// Identifies the CRL distribution point and scope for a particular CRL, and it indicates
/// whether the CRL covers revocation for end entity certificates only, CA certificates only,
/// attribute certificates only, or a limited set of reason codes.
#[derive(Debug, PartialEq)]
pub struct IssuingDistributionPoint<'bytes> {
  /// The distribution point name.
  pub distribution_point: Option<DistributionPointName<'bytes>>,
  /// Indicates whether the CRL only contains end entity certificates.
  pub only_contains_user_certs: Option<Boolean>,
  /// Indicates whether the CRL only contains CA certificates.
  pub only_contains_ca_certs: Option<Boolean>,
  /// Indicates which revocation reasons are covered by the CRL.
  pub only_some_reasons: Option<ReasonFlags>,
  /// Indicates whether the CRL is an indirect CRL.
  pub indirect_crl: Option<Boolean>,
  /// Indicates whether the CRL only contains attribute certificates.
  pub only_contains_attribute_certs: Option<Boolean>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for IssuingDistributionPoint<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidExtensionIssuingDistributionPoint.into());
    };
    dw.bytes = value;
    let distribution_point = Opt::decode(dw, DISTRIBUTION_POINT_TAG)?.0;
    let only_contains_user_certs = Opt::decode(dw, ONLY_CONTAINS_USER_CERTS_TAG)?.0;
    let only_contains_ca_certs = Opt::decode(dw, ONLY_CONTAINS_CA_CERTS_TAG)?.0;
    let only_some_reasons = Opt::decode(dw, ONLY_SOME_REASONS_TAG)?.0;
    let indirect_crl = Opt::decode(dw, INDIRECT_CRL_TAG)?.0;
    let only_contains_attribute_certs = Opt::decode(dw, ONLY_CONTAINS_ATTRIBUTE_CERTS_TAG)?.0;
    dw.bytes = rest;
    Ok(Self {
      distribution_point,
      only_contains_user_certs,
      only_contains_ca_certs,
      only_some_reasons,
      indirect_crl,
      only_contains_attribute_certs,
    })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for IssuingDistributionPoint<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG, |local_ew| {
      Opt(&self.distribution_point).encode(local_ew, DISTRIBUTION_POINT_TAG)?;
      Opt(&self.only_contains_user_certs).encode(local_ew, ONLY_CONTAINS_USER_CERTS_TAG)?;
      Opt(&self.only_contains_ca_certs).encode(local_ew, ONLY_CONTAINS_CA_CERTS_TAG)?;
      Opt(&self.only_some_reasons).encode(local_ew, ONLY_SOME_REASONS_TAG)?;
      Opt(&self.indirect_crl).encode(local_ew, INDIRECT_CRL_TAG)?;
      Opt(&self.only_contains_attribute_certs)
        .encode(local_ew, ONLY_CONTAINS_ATTRIBUTE_CERTS_TAG)?;
      Ok(())
    })
  }
}
