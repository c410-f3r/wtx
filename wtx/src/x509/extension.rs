use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, BOOLEAN_TAG, Boolean, Len, Octetstring, Oid,
    SEQUENCE_TAG, asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::X509Error,
};

/// Additional functionality.
#[derive(Debug, PartialEq)]
pub struct Extension<'bytes> {
  /// The OID that identifies the type of this extension.
  pub extn_id: Oid,
  /// Whether the extension is critical (defaults to false if absent in DER).
  pub critical: bool,
  /// Contains the DER encoding of an ASN.1 value corresponding to the extension type
  /// identified by `extn_id`.
  pub extn_value: Octetstring<&'bytes [u8]>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for Extension<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidExtension.into());
    };
    dw.bytes = value;
    let extn_id = Oid::decode(dw)?;
    let critical =
      if dw.bytes.first() == Some(&BOOLEAN_TAG) { Boolean::decode(dw)?.0 } else { false };
    let extn_value = Octetstring::decode(dw)?;
    dw.bytes = rest;
    Ok(Self { extn_id, critical, extn_value })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for Extension<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_THREE_BYTES, SEQUENCE_TAG, |local_ew| {
      self.extn_id.encode(local_ew)?;
      if self.critical {
        Boolean(self.critical).encode(local_ew)?;
      }
      self.extn_value.encode(local_ew)?;
      Ok(())
    })
  }
}
