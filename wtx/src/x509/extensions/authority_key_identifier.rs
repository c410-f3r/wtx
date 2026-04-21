use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, Len, Opt, SEQUENCE_TAG, asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::{
    AUTHORITY_CERT_ISSUER_TAG, AUTHORITY_CERT_SERIAL_NUMBER_TAG, GeneralNames, KEY_IDENTIFIER_TAG,
    KeyIdentifier, SerialNumber, X509Error,
  },
};

/// Provides a means of identifying the public key corresponding to the private key used to sign
/// a certificate.
#[derive(Debug, PartialEq)]
pub struct AuthorityKeyIdentifier {
  /// See [`KeyIdentifier`].
  pub key_identifier: Option<KeyIdentifier>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for AuthorityKeyIdentifier {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidExtensionAuthorityKeyIdentifier.into());
    };
    dw.bytes = value;
    let key_identifier = Opt::decode(dw, KEY_IDENTIFIER_TAG)?.0;
    dw.decode_aux.tag = Some(AUTHORITY_CERT_ISSUER_TAG);
    let authority_cert_issuer: Option<GeneralNames<'_>> =
      Opt::decode(dw, AUTHORITY_CERT_ISSUER_TAG)?.0;
    dw.decode_aux.tag = None;
    let authority_cert_serial_number: Option<SerialNumber> =
      Opt::decode(dw, AUTHORITY_CERT_SERIAL_NUMBER_TAG)?.0;
    if authority_cert_issuer.is_some() || authority_cert_serial_number.is_some() {
      return Err(X509Error::InvalidExtensionAuthorityKeyIdentifier.into());
    }
    dw.bytes = rest;
    Ok(Self { key_identifier })
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for AuthorityKeyIdentifier {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG, |local_ew| {
      Opt(&self.key_identifier).encode(local_ew, KEY_IDENTIFIER_TAG)
    })
  }
}
