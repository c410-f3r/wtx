use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Boolean, Len, Opt, SEQUENCE_TAG, asn1_writer,
    decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
  x509::{
    DISTRIBUTION_POINT_TAG, DistributionPointName, INDIRECT_CRL_TAG,
    ONLY_CONTAINS_ATTRIBUTE_CERTS_TAG, ONLY_CONTAINS_CA_CERTS_TAG, ONLY_CONTAINS_USER_CERTS_TAG,
    ONLY_SOME_REASONS_TAG, ReasonFlags, X509Error,
  },
};

/// Identifies the CRL distribution point and scope for a particular CRL, and it indicates
/// whether the CRL covers revocation for end entity certificates only, CA certificates only,
/// attribute certificates only, or a limited set of reason codes.
#[derive(Clone, Debug, PartialEq)]
pub struct IssuingDistributionPoint<B> {
  /// The distribution point name.
  pub distribution_point: Option<DistributionPointName<B>>,
  /// Indicates whether the CRL only contains end entity certificates.
  pub only_contains_user_certs: Option<bool>,
  /// Indicates whether the CRL only contains CA certificates.
  pub only_contains_ca_certs: Option<bool>,
  /// Indicates which revocation reasons are covered by the CRL.
  pub only_some_reasons: Option<ReasonFlags>,
  /// Indicates whether the CRL is an indirect CRL.
  pub indirect_crl: Option<bool>,
  /// Indicates whether the CRL only contains attribute certificates.
  pub only_contains_attribute_certs: Option<bool>,
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for IssuingDistributionPoint<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidExtensionIssuingDistributionPoint.into());
    };
    dw.bytes = value;
    let distribution_point = Opt::decode(dw, DISTRIBUTION_POINT_TAG)?.0;
    let only_contains_user_certs: Option<Boolean> =
      Opt::decode(dw, ONLY_CONTAINS_USER_CERTS_TAG)?.0;
    let only_contains_ca_certs: Option<Boolean> = Opt::decode(dw, ONLY_CONTAINS_CA_CERTS_TAG)?.0;
    let only_some_reasons = Opt::decode(dw, ONLY_SOME_REASONS_TAG)?.0;
    let indirect_crl: Option<Boolean> = Opt::decode(dw, INDIRECT_CRL_TAG)?.0;
    let only_contains_attribute_certs: Option<Boolean> =
      Opt::decode(dw, ONLY_CONTAINS_ATTRIBUTE_CERTS_TAG)?.0;
    dw.bytes = rest;
    Ok(Self {
      distribution_point,
      only_contains_user_certs: only_contains_user_certs.map(|el| el.0),
      only_contains_ca_certs: only_contains_ca_certs.map(|el| el.0),
      only_some_reasons,
      indirect_crl: indirect_crl.map(|el| el.0),
      only_contains_attribute_certs: only_contains_attribute_certs.map(|el| el.0),
    })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for IssuingDistributionPoint<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG, |local_ew| {
      Opt(&self.distribution_point).encode(local_ew, DISTRIBUTION_POINT_TAG)?;
      Opt(self.only_contains_user_certs.map(Boolean))
        .encode(local_ew, ONLY_CONTAINS_USER_CERTS_TAG)?;
      Opt(self.only_contains_ca_certs.map(Boolean)).encode(local_ew, ONLY_CONTAINS_CA_CERTS_TAG)?;
      Opt(&self.only_some_reasons).encode(local_ew, ONLY_SOME_REASONS_TAG)?;
      Opt(&self.indirect_crl.map(Boolean)).encode(local_ew, INDIRECT_CRL_TAG)?;
      Opt(&self.only_contains_attribute_certs.map(Boolean))
        .encode(local_ew, ONLY_CONTAINS_ATTRIBUTE_CERTS_TAG)?;
      Ok(())
    })
  }
}
