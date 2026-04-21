use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Octetstring},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::X509Error,
};
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Possible IP addresses in X.509
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IpAddress {
  /// IPv4 address
  Ipv4([u8; 4]),
  /// IPv4 address with subnet mask
  Ipv4WithMask([u8; 8]),
  /// IPv6 address
  Ipv6([u8; 16]),
  /// IPv6 address with subnet mask
  Ipv6WithMask([u8; 32]),
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for IpAddress {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Self::try_from(*Octetstring::decode(dw)?.bytes())
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for IpAddress {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    Octetstring::from_bytes(<&[u8]>::from(self)).encode(ew)
  }
}

impl From<IpAddr> for IpAddress {
  #[inline]
  fn from(value: IpAddr) -> Self {
    match value {
      IpAddr::V4(el) => IpAddress::Ipv4(el.octets()),
      IpAddr::V6(el) => IpAddress::Ipv6(el.octets()),
    }
  }
}

impl From<Ipv4Addr> for IpAddress {
  #[inline]
  fn from(value: Ipv4Addr) -> Self {
    IpAddress::Ipv4(value.octets())
  }
}

impl From<Ipv6Addr> for IpAddress {
  #[inline]
  fn from(value: Ipv6Addr) -> Self {
    IpAddress::Ipv6(value.octets())
  }
}

impl<'any> From<&'any IpAddress> for &'any [u8] {
  #[inline]
  fn from(value: &'any IpAddress) -> Self {
    match value {
      IpAddress::Ipv4(el) => el.as_slice(),
      IpAddress::Ipv6(el) => el.as_slice(),
      IpAddress::Ipv4WithMask(el) => el.as_slice(),
      IpAddress::Ipv6WithMask(el) => el.as_slice(),
    }
  }
}

impl<'bytes> TryFrom<&'bytes [u8]> for IpAddress {
  type Error = crate::Error;
  #[inline]
  fn try_from(bytes: &'bytes [u8]) -> Result<Self, Self::Error> {
    Ok(match bytes.len() {
      4 => Self::Ipv4(bytes.try_into().unwrap()),
      8 => Self::Ipv4WithMask(bytes.try_into().unwrap()),
      16 => Self::Ipv6(bytes.try_into().unwrap()),
      32 => Self::Ipv6WithMask(bytes.try_into().unwrap()),
      _ => return Err(X509Error::InvalidIpAddressRepresentation.into()),
    })
  }
}
