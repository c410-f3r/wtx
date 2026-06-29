use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Len, Oid, SEQUENCE_TAG, asn1_writer,
    decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
  x509::{GeneralName, X509Error},
};

/// The format and location of additional information provided by the subject of the certificate
/// in which this extension appears.
#[derive(Debug, PartialEq)]
pub struct AccessDescription<B> {
  /// See [`Oid`]
  pub access_method: Oid,
  /// See [`GeneralName`]
  pub access_location: GeneralName<B>,
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for AccessDescription<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidAccessDescription.into());
    };
    dw.bytes = value;
    let access_method = Oid::decode(dw)?;
    let access_location = GeneralName::decode(dw)?;
    dw.bytes = rest;
    Ok(Self { access_method, access_location })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for AccessDescription<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG, |local_ew| {
      self.access_method.encode(local_ew)?;
      self.access_location.encode(local_ew)?;
      Ok(())
    })
  }
}
