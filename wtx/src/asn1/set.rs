use crate::{
  asn1::{Asn1Error, Len, SET_TAG, asn1_writer, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  collection::TryExtend,
  misc::{Lease, SingleTypeStorage},
};

/// A collection of elements.
#[derive(Debug, PartialEq)]
pub struct Set<S>(
  /// A collection of elements.
  pub S,
);

impl<'de, S> Decode<'de, GenericCodec<Option<u8>, ()>> for Set<S>
where
  S: Default + SingleTypeStorage + TryExtend<[S::Item; 1]>,
  S::Item: Decode<'de, GenericCodec<Option<u8>, ()>>,
{
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Option<u8>>) -> crate::Result<Self> {
    let (SET_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(Asn1Error::InvalidSet.into());
    };
    let mut set = S::default();
    dw.bytes = value;
    while !dw.bytes.is_empty() {
      set.try_extend([S::Item::decode(dw)?])?;
    }
    dw.bytes = rest;
    Ok(Self(set))
  }
}

impl<S> Encode<GenericCodec<(), ()>> for Set<S>
where
  S: Lease<[S::Item]> + SingleTypeStorage,
  S::Item: Encode<GenericCodec<(), ()>>,
{
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, ()>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_ONE, SET_TAG, |local_ew| {
      for elem in self.0.lease() {
        elem.encode(local_ew)?;
      }
      Ok(())
    })
  }
}
