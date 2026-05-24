use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Len, SEQUENCE_TAG, SequenceBuffer},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collection::Vector,
  x509::RelativeDistinguishedName,
};

/// Distinguished name.
#[derive(Debug)]
pub struct Name<'bytes> {
  /// Bytes that compose all sequences
  bytes: &'bytes [u8],
  /// Entries
  rdn_sequence: Vector<RelativeDistinguishedName<'bytes>>,
}

impl<'bytes> Name<'bytes> {
  /// Shortcut
  #[inline]
  pub const fn new(
    bytes: &'bytes [u8],
    rdn_sequence: Vector<RelativeDistinguishedName<'bytes>>,
  ) -> Self {
    Self { bytes, rdn_sequence }
  }

  /// Bytes that compose all sequences
  #[inline]
  pub const fn bytes(&self) -> &'bytes [u8] {
    self.bytes
  }

  /// Entries
  #[inline]
  pub fn rdn_sequence(&self) -> &[RelativeDistinguishedName<'bytes>] {
    &self.rdn_sequence
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for Name<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (instance, bytes) = SequenceBuffer::decode(dw, SEQUENCE_TAG)?;
    Ok(Self { bytes, rdn_sequence: instance.0 })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for Name<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    SequenceBuffer(&self.rdn_sequence).encode(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG)
  }
}

impl<'bytes> PartialEq for Name<'bytes> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.bytes == other.bytes
  }
}
