use crate::misc::{Either, Lease, from_utf8_basic};
use core::net::IpAddr;

/// An IP address or a domain
#[derive(Debug, PartialEq)]
pub struct ServerName<D = ()>(
  /// See [`Either`].
  pub Either<IpAddr, D>,
);

impl<D> ServerName<D>
where
  D: Lease<[u8]>,
{
  #[inline]
  pub(crate) fn bytes<'ip, 'rslt, 'this>(&'this self, ip_buffer: &'ip mut [u8; 16]) -> &'rslt [u8]
  where
    'ip: 'rslt,
    'this: 'rslt,
  {
    match &self.0 {
      Either::Left(IpAddr::V4(el)) => {
        let [a, b, c, d] = el.octets();
        ip_buffer[0] = a;
        ip_buffer[1] = b;
        ip_buffer[2] = c;
        ip_buffer[3] = d;
        &ip_buffer[..4]
      }
      Either::Left(IpAddr::V6(el)) => {
        *ip_buffer = el.octets();
        &ip_buffer[..]
      }
      Either::Right(el) => el.lease(),
    }
  }
}
impl<'this> ServerName<&'this [u8]> {
  /// Tries to first convert `data` to [`IpAddr`]. If unsuccessful, fallbacks to a domain.
  pub fn from_ascii(data: &'this [u8]) -> crate::Result<Self> {
    if let Ok(ip_addr) = from_utf8_basic(data)?.parse() {
      return Ok(Self(Either::Left(ip_addr)));
    }
    Ok(Self(Either::Right(data)))
  }
}

impl<'this> ServerName<&'this str> {
  /// Tries to first convert `data` to [`IpAddr`]. If unsuccessful, fallbacks to a domain.
  pub fn from_arbitrary_str(data: &'this str) -> crate::Result<Self> {
    if let Ok(ip_addr) = data.parse() {
      return Ok(Self(Either::Left(ip_addr)));
    }
    Ok(Self(Either::Right(data)))
  }
}

impl<D> From<IpAddr> for ServerName<D> {
  #[inline]
  fn from(value: IpAddr) -> Self {
    Self(Either::Left(value))
  }
}

impl<'any> From<&'any [u8]> for ServerName<&'any [u8]> {
  #[inline]
  fn from(value: &'any [u8]) -> Self {
    Self(Either::Right(value))
  }
}

impl<'any> From<&'any str> for ServerName<&'any str> {
  #[inline]
  fn from(value: &'any str) -> Self {
    Self(Either::Right(value))
  }
}
