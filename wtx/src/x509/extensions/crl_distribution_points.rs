use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, Len, Opt, SEQUENCE_TAG, SequenceBuffer, asn1_writer,
    decode_asn1_tlv,
  },
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  collection::Vector,
  x509::{
    CRL_ISSUER_TAG, DISTRIBUTION_POINT_TAG, DistributionPointName, GeneralNames, REASONS_TAG,
    ReasonFlags, X509Error,
  },
};

/// Identifies how CRL information is obtained.
#[derive(Debug, PartialEq)]
pub struct CrlDistributionPoints<'bytes>(
  /// Identifies how CRL information is obtained.
  pub Vector<DistributionPoint<'bytes>>,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for CrlDistributionPoints<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self(SequenceBuffer::decode(dw, SEQUENCE_TAG)?.0))
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for CrlDistributionPoints<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    SequenceBuffer(&self.0).encode(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG)
  }
}

/// An entry in the CRL Distribution Points extension.
#[derive(Debug, PartialEq)]
pub struct DistributionPoint<'bytes> {
  /// See [`DistributionPointName`].
  pub distribution_point: Option<DistributionPointName<'bytes>>,
  /// See [`ReasonFlags`].
  pub reasons: Option<ReasonFlags>,
  /// See [`GeneralNames`].
  pub crl_issuer: Option<GeneralNames<'bytes>>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for DistributionPoint<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
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

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for DistributionPoint<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG, |local_ew| {
      Opt(&self.distribution_point).encode(local_ew, DISTRIBUTION_POINT_TAG)?;
      Opt(&self.reasons).encode(local_ew, REASONS_TAG)?;
      Opt(&self.crl_issuer).encode(local_ew, CRL_ISSUER_TAG)?;
      Ok(())
    })
  }
}
