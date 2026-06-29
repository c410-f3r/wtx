use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Len, Opt, SEQUENCE_TAG, SequenceBuffer,
    asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collections::Vector,
  misc::Lease,
  x509::{
    CRL_ISSUER_TAG, DISTRIBUTION_POINT_TAG, DistributionPointName, GeneralNames, REASONS_TAG,
    ReasonFlags, X509Error,
  },
};

/// Identifies how CRL information is obtained.
#[derive(Clone, Debug, PartialEq)]
pub struct CrlDistributionPoints<B> {
  /// Identifies how CRL information is obtained.
  pub entries: Vector<DistributionPoint<B>>,
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for CrlDistributionPoints<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    Ok(Self { entries: SequenceBuffer::decode(dw, SEQUENCE_TAG)?.0.0 })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for CrlDistributionPoints<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    SequenceBuffer(&self.entries).encode(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG)
  }
}

/// An entry in the CRL Distribution Points extension.
#[derive(Clone, Debug, PartialEq)]
pub struct DistributionPoint<B> {
  /// See [`DistributionPointName`].
  pub distribution_point: Option<DistributionPointName<B>>,
  /// See [`ReasonFlags`].
  pub reasons: Option<ReasonFlags>,
  /// See [`GeneralNames`].
  pub crl_issuer: Option<GeneralNames<B>>,
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for DistributionPoint<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidExtensionCrlDistributionPoints.into());
    };
    dw.bytes = value;
    let distribution_point = Opt::decode(dw, DISTRIBUTION_POINT_TAG)?.0;
    let reasons = Opt::decode(dw, REASONS_TAG)?.0;
    let crl_issuer = Opt::decode(dw, CRL_ISSUER_TAG)?.0;
    dw.bytes = rest;
    Ok(Self { distribution_point, reasons, crl_issuer })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for DistributionPoint<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG, |local_ew| {
      Opt(&self.distribution_point).encode(local_ew, DISTRIBUTION_POINT_TAG)?;
      Opt(&self.reasons).encode(local_ew, REASONS_TAG)?;
      Opt(&self.crl_issuer).encode(local_ew, CRL_ISSUER_TAG)?;
      Ok(())
    })
  }
}
