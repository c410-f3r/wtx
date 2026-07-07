use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Asn1Error, Len, OCTET_STRING_TAG, decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
};

/// Differently from `BitString`, each element occupies 8bits. Not to be confused with UTF-8
/// strings.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Octetstring<B> {
  bytes: B,
  tag: u8,
}

impl<B> Octetstring<B>
where
  B: Lease<[u8]>,
{
  /// Constructor
  #[inline]
  pub const fn from_bytes(bytes: B) -> Self {
    Self { bytes, tag: OCTET_STRING_TAG }
  }

  /// Raw bytes
  #[inline]
  pub const fn bytes(&self) -> &B {
    &self.bytes
  }

  /// Owned version of [`Self::bytes`].
  #[inline]
  pub fn into_bytes(self) -> B {
    self.bytes
  }
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for Octetstring<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (tag, _, value, rest) = decode_asn1_tlv(dw.bytes)?;
    if tag != dw.decode_aux.tag.unwrap_or(OCTET_STRING_TAG) {
      return Err(Asn1Error::InvalidOctetstring.into());
    }
    dw.bytes = rest;
    Ok(Self { bytes: value.try_into().map_err(Into::into)?, tag })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for Octetstring<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    let _ = ew.buffer.extend_from_copyable_slices([
      &[self.tag][..],
      &*Len::from_usize(0, self.bytes.lease().len())?,
      self.bytes.lease(),
    ])?;
    Ok(())
  }
}
