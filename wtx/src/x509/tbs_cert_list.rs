use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, INTEGER_TAG, Len, Opt, SEQUENCE_TAG, U32,
    asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
  x509::{
    AlgorithmIdentifier, EXPLICIT_TAG0, Extensions, Name, OptTime, RevokedCertificates, Time,
    X509Error,
  },
};

/// A sequence containing the name of the issuer, issue date, issue date of the next list, the
/// optional list of revoked certificates, and optional CRL extensions.
#[derive(Debug, PartialEq)]
pub struct TbsCertList<B>
where
  B: Lease<[u8]>,
{
  /// The entirety of the bytes that compose this structure.
  pub bytes: B,
  /// See [`AlgorithmIdentifier`].
  pub signature: AlgorithmIdentifier<B>,
  /// The issuer name identifies the entity that has signed and issued the CRL.
  pub issuer: Name<B>,
  /// Indicates the issue date of this CRL.
  pub this_update: Time,
  /// Indicates the date and time by which the CA will issue a new update, making this
  /// structure outdated.
  pub next_update: Option<Time>,
  /// See [`RevokedCertificates`].
  pub revoked_certificates: Option<RevokedCertificates<B>>,
  /// Additional information.
  pub crl_extensions: Option<Extensions<B>>,
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for TbsCertList<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
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
        return Err(X509Error::InvalidTbsCertListVersion.into());
      }
    }
    let signature = AlgorithmIdentifier::decode(dw)?;
    let issuer = Name::decode(dw)?;
    let this_update = Time::decode(dw)?;
    let next_update = OptTime::decode(dw)?.0;
    let revoked_certificates = Opt::decode(dw, SEQUENCE_TAG)?.0;
    let crl_extensions = Opt::decode(dw, EXPLICIT_TAG0)?.0;
    dw.bytes = rest;
    Ok(Self {
      bytes: bytes.try_into().map_err(Into::into)?,
      signature,
      issuer,
      this_update,
      next_update,
      revoked_certificates,
      crl_extensions,
    })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for TbsCertList<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
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
