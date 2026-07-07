use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, BitString, INTEGER_TAG, Len, Opt, SEQUENCE_TAG,
    asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
  x509::{
    AlgorithmIdentifier, EXPLICIT_TAG0, EXPLICIT_TAG3, Extensions, ISSUER_UID_TAG, Name,
    SUBJECT_UID_TAG, SerialNumber, SubjectPublicKeyInfo, Validity, X509Error,
  },
};

/// The "to be signed" portion of an X.509 certificate containing all certified fields.
#[derive(Debug, PartialEq)]
pub struct TbsCertificate<B>
where
  B: Lease<[u8]>,
{
  /// The unique serial number assigned by the issuing CA.
  pub serial_number: SerialNumber,
  /// The algorithm the CA used to sign this certificate.
  pub signature: AlgorithmIdentifier<B>,
  /// The distinguished name of the certificate issuer (the signing CA).
  pub issuer: Name<B>,
  /// See [`Validity`].
  pub validity: Validity,
  /// The distinguished name of the entity this certificate identifies.
  pub subject: Name<B>,
  /// The subject's public key and its associated algorithm.
  pub subject_public_key_info: SubjectPublicKeyInfo<B>,
  /// Optional issuer unique identifier.
  pub issuer_unique_id: Option<BitString<B>>,
  /// Optional subject unique identifier.
  pub subject_unique_id: Option<BitString<B>>,
  /// See [`Extensions`].
  pub extensions: Option<Extensions<B>>,
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for TbsCertificate<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let tbs_range_begin = dw.decode_aux.curr_idx;
    let (SEQUENCE_TAG, len, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidTbsCertificate.into());
    };

    let tbs_header_len = len.bytes().len().wrapping_add(1);
    let tbs_total_len = value.len().wrapping_add(tbs_header_len.into());
    let tbs_range_end = tbs_range_begin.wrapping_add(tbs_total_len);
    dw.decode_aux.tbs_cert_range = tbs_range_begin.try_into()?..tbs_range_end.try_into()?;

    dw.bytes = value;
    let (EXPLICIT_TAG0, _, [INTEGER_TAG, 1, 2], after) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidCertificateVersion.into());
    };
    dw.bytes = after;
    let serial_number = SerialNumber::decode(dw)?;
    let signature = AlgorithmIdentifier::decode(dw)?;
    let issuer = Name::decode(dw)?;
    let validity = Validity::decode(dw)?;
    let subject = Name::decode(dw)?;

    let spki_range_begin = tbs_range_end.wrapping_sub(dw.bytes.len());
    let subject_public_key_info = SubjectPublicKeyInfo::decode(dw)?;
    let spki_range_end = tbs_range_end.wrapping_sub(dw.bytes.len());
    dw.decode_aux.spki_range = spki_range_begin.try_into()?..spki_range_end.try_into()?;

    let issuer_unique_id = Opt::decode(dw, ISSUER_UID_TAG)?.0;
    let subject_unique_id = Opt::decode(dw, SUBJECT_UID_TAG)?.0;
    let extensions = Opt::decode(dw, EXPLICIT_TAG3)?.0;
    dw.bytes = rest;

    Ok(Self {
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

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for TbsCertificate<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
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
