use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, BitString, INTEGER_TAG, Len, Opt, SEQUENCE_TAG,
    asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::{
    AlgorithmIdentifier, EXPLICIT_TAG0, EXPLICIT_TAG3, Extensions, ISSUER_UID_TAG, Name,
    SUBJECT_UID_TAG, SerialNumber, SubjectPublicKeyInfo, Validity, X509Error,
  },
};

/// The "to be signed" portion of an X.509 certificate containing all certified fields.
#[derive(Debug, PartialEq)]
pub struct TbsCertificate<'bytes> {
  /// The entirety of the bytes that compose this structure.
  pub bytes: &'bytes [u8],
  /// The unique serial number assigned by the issuing CA.
  pub serial_number: SerialNumber,
  /// The algorithm the CA used to sign this certificate.
  pub signature: AlgorithmIdentifier<'bytes>,
  /// The distinguished name of the certificate issuer (the signing CA).
  pub issuer: Name<'bytes>,
  /// See [`Validity`].
  pub validity: Validity,
  /// The distinguished name of the entity this certificate identifies.
  pub subject: Name<'bytes>,
  /// The subject's public key and its associated algorithm.
  pub subject_public_key_info: SubjectPublicKeyInfo<'bytes>,
  /// Optional issuer unique identifier.
  pub issuer_unique_id: Option<BitString<&'bytes [u8]>>,
  /// Optional subject unique identifier.
  pub subject_unique_id: Option<BitString<&'bytes [u8]>>,
  /// See [`Extensions`].
  pub extensions: Option<Extensions<'bytes, EXPLICIT_TAG3>>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for TbsCertificate<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let full_bytes = dw.bytes;
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidTbsCertificate.into());
    };
    let bytes = {
      let idx = full_bytes.len().wrapping_sub(rest.len());
      full_bytes.get(..idx).unwrap_or_default()
    };
    dw.bytes = value;
    let (EXPLICIT_TAG0, _, [INTEGER_TAG, 1, 2], after) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidVersion.into());
    };
    dw.bytes = after;
    let serial_number = SerialNumber::decode(dw)?;
    let signature = AlgorithmIdentifier::decode(dw)?;
    let issuer = Name::decode(dw)?;
    let validity = Validity::decode(dw)?;
    let subject = Name::decode(dw)?;
    let subject_public_key_info = SubjectPublicKeyInfo::decode(dw)?;
    let issuer_unique_id = Opt::decode(dw, ISSUER_UID_TAG)?.0;
    let subject_unique_id = Opt::decode(dw, SUBJECT_UID_TAG)?.0;
    let extensions = Opt::decode(dw, EXPLICIT_TAG3)?.0;
    dw.bytes = rest;
    Ok(Self {
      bytes,
      serial_number,
      signature,
      issuer,
      validity,
      subject,
      subject_public_key_info,
      issuer_unique_id,
      subject_unique_id,
      extensions,
    })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for TbsCertificate<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_THREE_BYTES, SEQUENCE_TAG, |local_ew| {
      let version_bytes = [&[EXPLICIT_TAG0, 3, INTEGER_TAG, 1, 2][..]];
      let _ = local_ew.buffer.extend_from_copyable_slices(version_bytes)?;
      self.serial_number.encode(local_ew)?;
      self.signature.encode(local_ew)?;
      self.issuer.encode(local_ew)?;
      self.validity.encode(local_ew)?;
      self.subject.encode(local_ew)?;
      self.subject_public_key_info.encode(local_ew)?;
      Opt(&self.issuer_unique_id).encode(local_ew, ISSUER_UID_TAG)?;
      Opt(&self.subject_unique_id).encode(local_ew, SUBJECT_UID_TAG)?;
      Opt(&self.extensions).encode(local_ew, EXPLICIT_TAG3)?;
      Ok(())
    })
  }
}
