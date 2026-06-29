use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Octetstring},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
};

/// `RFC-9162` extension. The internal bytes are meant to be decoded by external actors.
#[derive(Debug, PartialEq)]
pub struct SignedCertificateTimestampList<B>(
  /// See [`Octetstring`].
  pub Octetstring<B>,
);

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>>
  for SignedCertificateTimestampList<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    Ok(Self(Octetstring::decode(dw)?))
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for SignedCertificateTimestampList<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    self.0.encode(ew)
  }
}
