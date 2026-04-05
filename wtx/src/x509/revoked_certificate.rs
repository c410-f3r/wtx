use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, Integer, Len, Opt, SEQUENCE_TAG, asn1_writer,
    decode_asn1_tlv,
  },
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::{Extensions, Time, X509Error},
};

/// A revoked certificate entry in a Certificate Revocation List (CRL)
#[derive(Debug, PartialEq)]
pub struct RevokedCertificate<'bytes> {
  /// Serial number of the revoked certificate.
  pub user_certificate: Integer<&'bytes [u8]>,
  /// The date and time when the certificate was revoked.
  pub revocation_date: Time,
  /// Additional information.
  pub crl_entry_extensions: Option<Extensions<'bytes, SEQUENCE_TAG>>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for RevokedCertificate<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidRevokedCertificate.into());
    };
    dw.bytes = value;
    let user_certificate = Integer::decode(dw)?;
    let revocation_date = Time::decode(dw)?;
    let crl_entry_extensions = Opt::decode(dw, SEQUENCE_TAG)?.0;
    dw.bytes = rest;
    Ok(Self { crl_entry_extensions, revocation_date, user_certificate })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for RevokedCertificate<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG, |local_ew| {
      self.user_certificate.encode(local_ew)?;
      self.revocation_date.encode(local_ew)?;
      Opt(&self.crl_entry_extensions).encode(local_ew, SEQUENCE_TAG)?;
      Ok(())
    })
  }
}
