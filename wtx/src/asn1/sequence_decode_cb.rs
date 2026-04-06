use core::marker::PhantomData;

use crate::{
  asn1::{Asn1DecodeWrapper, Asn1Error, decode_asn1_tlv},
  codec::{Decode, GenericCodec, GenericDecodeWrapper},
};

/// Helper that streams decoded elements to `C`
#[derive(Debug, PartialEq)]
pub struct SequenceDecodeCb<C, E>(C, PhantomData<E>);

impl<'de, C, E> SequenceDecodeCb<C, E>
where
  C: FnMut(E) -> crate::Result<()>,
  E: Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>>,
{
  /// Constructor.
  #[inline]
  pub const fn new(cb: C) -> Self {
    Self(cb, PhantomData)
  }

  /// The encoding of an collection object requires the injection of a tag.
  #[inline]
  pub fn decode(
    &mut self,
    dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>,
    tag: u8,
  ) -> crate::Result<()> {
    let (local_tag, _, value, rest) = decode_asn1_tlv(dw.bytes)?;
    if local_tag != tag {
      return Err(Asn1Error::InvalidGenericSequence(local_tag, tag).into());
    }
    dw.bytes = value;
    (self.0)(E::decode(dw)?)?;
    dw.bytes = rest;
    Ok(())
  }
}
