use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, ImplicitOpt, Len, SEQUENCE_TAG, asn1_writer,
    decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::{
    AUTHORITY_CERT_ISSUER_TAG, AUTHORITY_CERT_SERIAL_NUMBER_TAG, GeneralNames, KEY_IDENTIFIER_TAG,
    KeyIdentifier, SerialNumber, X509Error,
  },
};

/// Provides a means of identifying the public key corresponding to the private key used to sign
/// a certificate.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthorityKeyIdentifier {
  /// See [`KeyIdentifier`].
  pub key_identifier: Option<KeyIdentifier>,
}

impl AuthorityKeyIdentifier {
  /// Shortcut
  #[inline]
  pub const fn new(key_identifier: Option<KeyIdentifier>) -> Self {
    Self { key_identifier }
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for AuthorityKeyIdentifier {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidExtensionAuthorityKeyIdentifier.into());
    };
    dw.bytes = value;
    let key_identifier = ImplicitOpt::<_, KEY_IDENTIFIER_TAG>::decode(dw)?.0;
    let authority_cert_issuer: Option<GeneralNames<&[u8]>> =
      ImplicitOpt::<_, AUTHORITY_CERT_ISSUER_TAG>::decode(dw)?.0;
    let authority_cert_serial_number: Option<SerialNumber> =
      ImplicitOpt::<_, AUTHORITY_CERT_SERIAL_NUMBER_TAG>::decode(dw)?.0;
    if authority_cert_issuer.is_some() || authority_cert_serial_number.is_some() {
      return Err(X509Error::InvalidExtensionAuthorityKeyIdentifier.into());
    }
    dw.bytes = rest;
    Ok(Self { key_identifier })
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for AuthorityKeyIdentifier {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG, |local_ew| {
      ImplicitOpt::<_, KEY_IDENTIFIER_TAG>(&self.key_identifier).encode(local_ew)
    })
  }
}
