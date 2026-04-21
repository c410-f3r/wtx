use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Asn1Error, Len, OCTET_STRING_TAG, decode_asn1_tlv},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
};

/// Differently from `BitString`, each element occupies 8bits. Not to be confused with UTF-8
/// strings.
#[derive(Debug, PartialEq)]
pub struct Octetstring<S> {
  bytes: S,
  tag: u8,
}

impl<S> Octetstring<S>
where
  S: Lease<[u8]>,
{
  /// Constructor
  #[inline]
  pub const fn from_bytes(bytes: S) -> Self {
    Self { bytes, tag: OCTET_STRING_TAG }
  }

  /// String value
  #[inline]
  pub const fn bytes(&self) -> &S {
    &self.bytes
  }
}
// 48, 22, 128, 20, 116, 92, 209, 13, 86, 1, 79, 202, 101, 243, 150, 94, 201, 182, 0, 81, 148, 15, 232, 203
impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for Octetstring<&'de [u8]> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (tag, _, value, rest) = decode_asn1_tlv(dw.bytes)?;
    if tag != dw.decode_aux.tag.unwrap_or(OCTET_STRING_TAG) {
      return Err(Asn1Error::InvalidOctetstring.into());
    }
    dw.bytes = rest;
    Ok(Self { bytes: value, tag })
  }
}

impl<S> Encode<GenericCodec<(), Asn1EncodeWrapper>> for Octetstring<S>
where
  S: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    let _ = ew.buffer.extend_from_copyable_slices([
      &[self.tag][..],
      &*Len::from_usize(0, self.bytes.lease().len())?,
      self.bytes.lease(),
    ])?;
    Ok(())
  }
}
