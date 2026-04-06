use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, INTEGER_TAG, Len, Opt, SEQUENCE_TAG, U32, asn1_writer,
    decode_asn1_tlv,
  },
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::{
    AlgorithmIdentifier, EXPLICIT_TAG0, Extensions, Name, OptTime, RevokedCertificates, Time,
    X509Error,
  },
};

/// A sequence containing the name of the issuer, issue date, issue date of the next list, the
/// optional list of revoked certificates, and optional CRL extensions.
///
/// Only supports version 2.
#[derive(Debug, PartialEq)]
pub struct TbsCertList<'bytes> {
  /// The entirety of the bytes that compose this structure.
  pub bytes: &'bytes [u8],
  /// See [`AlgorithmIdentifier`].
  pub signature: AlgorithmIdentifier<'bytes>,
  /// The issuer name identifies the entity that has signed and issued the CRL.
  pub issuer: Name<'bytes>,
  /// Indicates the issue date of this CRL.
  pub this_update: Time,
  /// The date by which the next CRL will be issued.
  pub next_update: Option<Time>,
  /// See [`RevokedCertificates`].
  pub revoked_certificates: Option<RevokedCertificates<'bytes>>,
  /// Additional information.
  pub crl_extensions: Option<Extensions<'bytes, EXPLICIT_TAG0>>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for TbsCertList<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let full_bytes = dw.bytes;
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidTbsCertList.into());
    };
    let bytes = {
      let idx = full_bytes.len().wrapping_sub(rest.len());
      full_bytes.get(..idx).unwrap_or_default()
    };
    dw.bytes = value;
    if dw.bytes.first() == Some(&INTEGER_TAG) {
      let version = U32::decode(dw)?;
      if version != U32::ONE {
        return Err(X509Error::InvalidVersion.into());
      }
    } else {
      return Err(X509Error::InvalidVersion.into());
    }
    let signature = AlgorithmIdentifier::decode(dw)?;
    let issuer = Name::decode(dw)?;
    let this_update = Time::decode(dw)?;
    let next_update = OptTime::decode(dw)?.0;
    let revoked_certificates = Opt::decode(dw, SEQUENCE_TAG)?.0;
    let crl_extensions = Opt::decode(dw, EXPLICIT_TAG0)?.0;
    dw.bytes = rest;
    Ok(Self {
      bytes,
      signature,
      issuer,
      this_update,
      next_update,
      revoked_certificates,
      crl_extensions,
    })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for TbsCertList<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_THREE_BYTES, SEQUENCE_TAG, |local_ew| {
      U32::ONE.encode(local_ew)?;
      self.signature.encode(local_ew)?;
      self.issuer.encode(local_ew)?;
      self.this_update.encode(local_ew)?;
      OptTime(self.next_update).encode(local_ew)?;
      Opt(&self.revoked_certificates).encode(local_ew, SEQUENCE_TAG)?;
      Opt(&self.crl_extensions).encode(local_ew, EXPLICIT_TAG0)?;
      Ok(())
    })
  }
}
