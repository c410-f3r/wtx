use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Octetstring},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
};

/// `RFC-9162` extension. The internal bytes are meant to be decoded by external actors.
#[derive(Debug, PartialEq)]
pub struct SignedCertificateTimestampList<'bytes>(
  /// See [`Octetstring`].
  pub Octetstring<&'bytes [u8]>,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for SignedCertificateTimestampList<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self(Octetstring::decode(dw)?))
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>>
  for SignedCertificateTimestampList<'bytes>
{
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    self.0.encode(ew)
  }
}
