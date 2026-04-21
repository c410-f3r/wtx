use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, Len, SEQUENCE_TAG, SequenceBuffer, asn1_writer,
    decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collection::Vector,
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
#[derive(Debug, PartialEq)]
pub enum GeneralName<'bytes> {
  /// An OtherName (Opaque).
  OtherName(&'bytes [u8]),
  /// An RFC 822 email address (IA5String).
  Rfc822Name(&'bytes [u8]),
  /// A DNS domain name (IA5String).
  DnsName(&'bytes [u8]),
  /// An X.400 address (Opaque).
  X400Address(&'bytes [u8]),
  /// A directory name (DER).
  DirectoryName(&'bytes [u8]),
  /// An EDI party name (Opaque).
  EdiPartyName(&'bytes [u8]),
  /// A URI (IA5String).
  UniformResourceIdentifier(&'bytes [u8]),
  /// An IP address (4 bytes for IPv4, 16 for IPv6; 8 or 32 in name constraints).
  IpAddress(&'bytes [u8]),
  /// A registered OID
  RegisteredId(&'bytes [u8]),
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for GeneralName<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (tag, _, value, rest) = decode_asn1_tlv(dw.bytes)?;
    let name = (tag, value).try_into()?;
    dw.bytes = rest;
    Ok(name)
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for GeneralName<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    let (tag, content) = self.into();
    asn1_writer(ew, Len::MAX_TWO_BYTES, tag, |local_ew| {
      let _ = local_ew.buffer.extend_from_copyable_slices([content])?;
      Ok(())
    })
  }
}

impl<'bytes> From<&GeneralName<'bytes>> for (u8, &'bytes [u8]) {
  #[inline]
  fn from(value: &GeneralName<'bytes>) -> Self {
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

impl<'bytes> TryFrom<(u8, &'bytes [u8])> for GeneralName<'bytes> {
  type Error = crate::Error;

  #[inline]
  fn try_from((tag, value): (u8, &'bytes [u8])) -> Result<Self, Self::Error> {
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
#[derive(Debug, PartialEq)]
pub struct GeneralNames<'bytes> {
  /// Entries
  pub entries: Vector<GeneralName<'bytes>>,
  /// Tag
  pub tag: u8,
}

impl<'bytes> GeneralNames<'bytes> {
  /// If `None`, `tag` will be turned into the default sequence tag.
  pub fn new(entries: Vector<GeneralName<'bytes>>, tag: Option<u8>) -> Self {
    Self { entries, tag: tag.unwrap_or(SEQUENCE_TAG) }
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for GeneralNames<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let tag = dw.decode_aux.tag.unwrap_or(SEQUENCE_TAG);
    Ok(Self { entries: SequenceBuffer::decode(dw, tag)?.0, tag })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for GeneralNames<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    SequenceBuffer(&self.entries).encode(ew, Len::MAX_TWO_BYTES, self.tag)
  }
}
