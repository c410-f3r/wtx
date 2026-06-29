use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Len, SEQUENCE_TAG, SequenceBuffer, asn1_writer,
    decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collections::Vector,
  misc::Lease,
  x509::X509Error,
};

const OTHER_NAME_TAG: u8 = 160;
const RFC822_NAME_TAG: u8 = 129;
const DNS_NAME_TAG: u8 = 130;
const X400_ADDRESS_TAG: u8 = 163;
const DIRECTORY_NAME_TAG: u8 = 164;
const EDI_PARTY_NAME_TAG: u8 = 165;
const URI_TAG: u8 = 134;
const IP_ADDRESS_TAG: u8 = 135;
const REGISTERED_ID_TAG: u8 = 136;

/// Represents a name in one of several forms as defined in RFC 5280.
#[derive(Clone, Debug, PartialEq)]
pub enum GeneralName<B> {
  /// Other
  OtherName(B),
  /// Email address
  Rfc822Name(B),
  /// DNS domain name
  DnsName(B),
  /// X.400 address
  X400Address(B),
  /// Directory name
  DirectoryName(B),
  /// EDI party name
  EdiPartyName(B),
  /// URI
  UniformResourceIdentifier(B),
  /// IP address
  IpAddress(B),
  /// Oid
  RegisteredId(B),
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for GeneralName<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (tag, _, value, rest) = decode_asn1_tlv(dw.bytes)?;
    let name = (tag, value.try_into().map_err(Into::into)?).try_into()?;
    dw.bytes = rest;
    Ok(name)
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for GeneralName<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    let (tag, content) = self.into();
    asn1_writer(ew, Len::MAX_TWO_BYTES, tag, |local_ew| {
      let _ = local_ew.buffer.extend_from_copyable_slices([content])?;
      Ok(())
    })
  }
}

impl<'any, B> From<&'any GeneralName<B>> for (u8, &'any B) {
  #[inline]
  fn from(value: &'any GeneralName<B>) -> Self {
    match value {
      GeneralName::OtherName(el) => (OTHER_NAME_TAG, el),
      GeneralName::Rfc822Name(el) => (RFC822_NAME_TAG, el),
      GeneralName::DnsName(el) => (DNS_NAME_TAG, el),
      GeneralName::X400Address(el) => (X400_ADDRESS_TAG, el),
      GeneralName::DirectoryName(el) => (DIRECTORY_NAME_TAG, el),
      GeneralName::EdiPartyName(el) => (EDI_PARTY_NAME_TAG, el),
      GeneralName::UniformResourceIdentifier(el) => (URI_TAG, el),
      GeneralName::IpAddress(el) => (IP_ADDRESS_TAG, el),
      GeneralName::RegisteredId(el) => (REGISTERED_ID_TAG, el),
    }
  }
}

impl<B> TryFrom<(u8, B)> for GeneralName<B> {
  type Error = crate::Error;

  #[inline]
  fn try_from((tag, value): (u8, B)) -> Result<Self, Self::Error> {
    Ok(match tag {
      OTHER_NAME_TAG => Self::OtherName(value),
      RFC822_NAME_TAG => Self::Rfc822Name(value),
      DNS_NAME_TAG => Self::DnsName(value),
      X400_ADDRESS_TAG => Self::X400Address(value),
      DIRECTORY_NAME_TAG => Self::DirectoryName(value),
      EDI_PARTY_NAME_TAG => Self::EdiPartyName(value),
      URI_TAG => Self::UniformResourceIdentifier(value),
      IP_ADDRESS_TAG => Self::IpAddress(value),
      REGISTERED_ID_TAG => Self::RegisteredId(value),
      _ => return Err(X509Error::InvalidGeneralName.into()),
    })
  }
}

/// A sequence of [`GeneralName`] values.
#[derive(Clone, Debug, PartialEq)]
pub struct GeneralNames<B> {
  /// Entries
  pub entries: Vector<GeneralName<B>>,
  /// Tag
  pub tag: u8,
}

impl<B> GeneralNames<B> {
  /// If `None`, `tag` will be turned into the default sequence tag.
  #[inline]
  pub fn new(entries: Vector<GeneralName<B>>, tag: Option<u8>) -> Self {
    Self { entries, tag: tag.unwrap_or(SEQUENCE_TAG) }
  }
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for GeneralNames<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let tag = dw.decode_aux.tag.unwrap_or(SEQUENCE_TAG);
    Ok(Self { entries: SequenceBuffer::decode(dw, tag)?.0.0, tag })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for GeneralNames<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    SequenceBuffer(&self.entries).encode(ew, Len::MAX_TWO_BYTES, self.tag)
  }
}
