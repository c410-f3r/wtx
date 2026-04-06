use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, Len, Opt, SEQUENCE_TAG, asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::{
    AUTHORITY_CERT_ISSUER_TAG, AUTHORITY_CERT_SERIAL_NUMBER_TAG, GeneralNames, KEY_IDENTIFIER_TAG,
    KeyIdentifier, SerialNumber, X509Error,
  },
};

/// Provides a means of identifying the public key corresponding to the private key used to sign
/// a certificate.
#[derive(Debug, PartialEq)]
pub struct AuthorityKeyIdentifier<'bytes> {
  /// See [`KeyIdentifier`].
  pub key_identifier: Option<KeyIdentifier>,
  /// See [`GeneralNames`].
  pub authority_cert_issuer: Option<GeneralNames<'bytes>>,
  /// The serial number of the issuing CA's certificate.
  pub authority_cert_serial_number: Option<SerialNumber>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for AuthorityKeyIdentifier<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidExtensionAuthorityKeyIdentifier.into());
    };
    dw.bytes = value;
    let key_identifier = Opt::decode(dw, KEY_IDENTIFIER_TAG)?.0;
    let authority_cert_issuer = Opt::decode(dw, AUTHORITY_CERT_ISSUER_TAG)?.0;
    let authority_cert_serial_number = Opt::decode(dw, AUTHORITY_CERT_SERIAL_NUMBER_TAG)?.0;
    dw.bytes = rest;
    Ok(Self { key_identifier, authority_cert_issuer, authority_cert_serial_number })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for AuthorityKeyIdentifier<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG, |local_ew| {
      Opt(&self.key_identifier).encode(local_ew, KEY_IDENTIFIER_TAG)?;
      Opt(&self.authority_cert_issuer).encode(local_ew, AUTHORITY_CERT_ISSUER_TAG)?;
      Opt(&self.authority_cert_serial_number).encode(local_ew, AUTHORITY_CERT_SERIAL_NUMBER_TAG)?;
      Ok(())
    })
  }
}
