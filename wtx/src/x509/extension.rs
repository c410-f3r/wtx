use crate::{
  asn1::{BOOLEAN_TAG, Boolean, Len, Octetstring, Oid, SEQUENCE_TAG, asn1_writer, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::X509Error,
};

/// Additional functionality.
#[derive(Debug, PartialEq)]
pub struct Extension<'bytes> {
  /// The OID that identifies the type of this extension.
  pub extn_id: Oid,
  /// Whether the extension is critical (defaults to false if absent in DER).
  pub critical: Boolean,
  /// Contains the DER encoding of an ASN.1 value corresponding to the extension type
  /// identified by `extn_id`.
  pub extn_value: Octetstring<&'bytes [u8]>,
}

impl<'de> Decode<'de, GenericCodec<Option<u8>, ()>> for Extension<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Option<u8>>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidExtension.into());
    };
    dw.bytes = value;
    let extn_id = Oid::decode(dw)?;
    let critical =
      if dw.bytes.first() == Some(&BOOLEAN_TAG) { Boolean::decode(dw)? } else { Boolean(false) };
    let extn_value = Octetstring::decode(dw)?;
    dw.bytes = rest;
    Ok(Self { extn_id, critical, extn_value })
  }
}

impl<'bytes> Encode<GenericCodec<(), ()>> for Extension<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, ()>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_TWO, SEQUENCE_TAG, |local_ew| {
      self.extn_id.encode(local_ew)?;
      if self.critical.0 {
        self.critical.encode(local_ew)?;
      }
      self.extn_value.encode(local_ew)?;
      Ok(())
    })
  }
}
