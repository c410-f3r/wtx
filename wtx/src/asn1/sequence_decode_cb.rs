use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1Error, decode_asn1_tlv},
  codec::{Decode, DecodeWrapper, GenericCodec},
};
use core::marker::PhantomData;

/// Helper that streams decoded elements to `C`
#[derive(Debug, PartialEq)]
pub struct SequenceDecodeCb<C, E>(C, PhantomData<E>);

impl<'de, C, E> SequenceDecodeCb<C, E>
where
  C: FnMut(E) -> crate::Result<()>,
  E: Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>>,
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
    dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>,
    tag: u8,
  ) -> crate::Result<&'de [u8]> {
    let (local_tag, _, value, rest) = decode_asn1_tlv(dw.bytes)?;
    if local_tag != tag {
      return Err(Asn1Error::InvalidGenericSequence(local_tag, tag).into());
    }
    dw.bytes = value;
    while !dw.bytes.is_empty() {
      (self.0)(E::decode(dw)?)?;
    }
    dw.bytes = rest;
    Ok(value)
  }
}
