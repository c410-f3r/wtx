use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Asn1Error, Len, OCTET_STRING_TAG, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  misc::Lease,
};

/// Differently from `BitString`, each element occupies 8bits. Not to be confused with UTF-8
/// strings.
#[derive(Debug, PartialEq)]
pub struct Octetstring<S>(S);

impl<S> Octetstring<S>
where
  S: Lease<[u8]>,
{
  /// Constructor that only accepts strings
  #[inline]
  pub const fn new(string: S) -> Self {
    Self(string)
  }

  /// String value
  #[inline]
  pub const fn bytes(&self) -> &S {
    &self.0
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for Octetstring<&'de [u8]> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (OCTET_STRING_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(Asn1Error::InvalidOctetstring.into());
    };
    dw.bytes = rest;
    Ok(Self(value))
  }
}

impl<S> Encode<GenericCodec<(), Asn1EncodeWrapper>> for Octetstring<S>
where
  S: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    let _ = ew.buffer.extend_from_copyable_slices([
      &[OCTET_STRING_TAG][..],
      &*Len::from_usize(0, self.0.lease().len())?,
      self.0.lease(),
    ])?;
    Ok(())
  }
}
