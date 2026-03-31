use crate::{
  asn1::{
    BitString, EXTENSIONS_TAG, INTEGER_TAG, ISSUER_UID_TAG, Integer, Len, SEQUENCE_TAG,
    SUBJECT_UID_TAG, VERSION_TAG, asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  collection::Vector,
  x509::{AlgorithmIdentifier, Extension, Name, SubjectPublicKeyInfo, Validity, X509Error},
};

/// The "to be signed" portion of an X.509 certificate containing all certified fields.
#[derive(Debug, PartialEq)]
pub struct TbsCertificate<'bytes> {
  /// The unique serial number assigned by the issuing CA.
  pub serial_number: Integer<&'bytes [u8]>,
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
  /// See [`Extension`].
  pub extensions: Option<Vector<Extension<'bytes>>>,
}

impl<'de> Decode<'de, GenericCodec<Option<u8>, ()>> for TbsCertificate<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Option<u8>>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidTbsCertificate.into());
    };
    dw.bytes = value;
    if dw.bytes.first() == Some(&VERSION_TAG) {
      let (_, _, [INTEGER_TAG, 1, 2], after) = decode_asn1_tlv(dw.bytes)? else {
        return Err(X509Error::InvalidVersion.into());
      };
      dw.bytes = after;
    }
    let serial_number = Integer::decode(dw)?;
    let signature = AlgorithmIdentifier::decode(dw)?;
    let issuer = Name::decode(dw)?;
    let validity = Validity::decode(dw)?;
    let subject = Name::decode(dw)?;
    let subject_public_key_info = SubjectPublicKeyInfo::decode(dw)?;
    let issuer_unique_id = if dw.bytes.first() == Some(&ISSUER_UID_TAG) {
      dw.decode_aux = Some(ISSUER_UID_TAG);
      let ret = Some(BitString::decode(dw)?);
      dw.decode_aux = None;
      ret
    } else {
      None
    };
    let subject_unique_id = if dw.bytes.first() == Some(&SUBJECT_UID_TAG) {
      dw.decode_aux = Some(SUBJECT_UID_TAG);
      let ret = Some(BitString::decode(dw)?);
      dw.decode_aux = None;
      ret
    } else {
      None
    };
    let extensions = if dw.bytes.first() == Some(&EXTENSIONS_TAG) {
      let (_, _, ext_wrapper, after) = decode_asn1_tlv(dw.bytes)?;
      dw.bytes = ext_wrapper;
      let (SEQUENCE_TAG, _, seq_content, _) = decode_asn1_tlv(dw.bytes)? else {
        return Err(X509Error::InvalidTbsCertificate.into());
      };
      dw.bytes = seq_content;
      let mut exts = Vector::default();
      while !dw.bytes.is_empty() {
        exts.push(Extension::decode(dw)?)?;
      }
      dw.bytes = after;
      Some(exts)
    } else {
      None
    };
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

impl<'bytes> Encode<GenericCodec<(), ()>> for TbsCertificate<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, ()>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_THREE, SEQUENCE_TAG, |local_ew| {
      let version_bytes = [&[VERSION_TAG, 3, INTEGER_TAG, 1, 2][..]];
      let _ = local_ew.buffer.extend_from_copyable_slices(version_bytes)?;
      self.serial_number.encode(local_ew)?;
      self.signature.encode(local_ew)?;
      self.issuer.encode(local_ew)?;
      self.validity.encode(local_ew)?;
      self.subject.encode(local_ew)?;
      self.subject_public_key_info.encode(local_ew)?;
      if let Some(uid) = &self.issuer_unique_id {
        uid.encode(local_ew)?;
      }
      if let Some(uid) = &self.subject_unique_id {
        uid.encode(local_ew)?;
      }
      if let Some(exts) = &self.extensions {
        asn1_writer(local_ew, Len::MAX_THREE, EXTENSIONS_TAG, |ext_ew| {
          asn1_writer(ext_ew, Len::MAX_THREE, SEQUENCE_TAG, |seq_ew| {
            for ext in exts.iter() {
              ext.encode(seq_ew)?;
            }
            Ok(())
          })
        })?;
      }
      Ok(())
    })
  }
}
