use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Len, SEQUENCE_TAG, SequenceBuffer},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collections::Vector,
  misc::Lease,
  x509::RelativeDistinguishedName,
};

/// Distinguished name.
#[derive(Clone, Debug, Default)]
pub struct Name<B> {
  /// Bytes that compose all sequences
  bytes: B,
  /// Entries
  rdn_sequence: Vector<RelativeDistinguishedName<B>>,
}

impl<B> Name<B> {
  /// Shortcut
  #[inline]
  pub const fn new(bytes: B, rdn_sequence: Vector<RelativeDistinguishedName<B>>) -> Self {
    Self { bytes, rdn_sequence }
  }

  /// Bytes that compose all sequences
  #[inline]
  pub const fn bytes(&self) -> &B {
    &self.bytes
  }

  /// Returns the inner elements
  #[inline]
  pub fn into_parts(self) -> (B, Vector<RelativeDistinguishedName<B>>) {
    (self.bytes, self.rdn_sequence)
  }

  /// Entries
  #[inline]
  pub fn rdn_sequence(&self) -> &[RelativeDistinguishedName<B>] {
    &self.rdn_sequence
  }
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for Name<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (instance, bytes) = SequenceBuffer::decode(dw, SEQUENCE_TAG)?;
    Ok(Self { bytes: bytes.try_into().map_err(Into::into)?, rdn_sequence: instance.0 })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for Name<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    SequenceBuffer(&self.rdn_sequence).encode(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG)
  }
}

impl<B> PartialEq for Name<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.bytes.lease() == other.bytes.lease()
  }
}
