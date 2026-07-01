use crate::{
  codec::{Decode, Encode},
  database::{
    Typed,
    client::postgres::{Postgres, PostgresDecodeWrapper, PostgresEncodeWrapper, PostgresError, Ty},
  },
};
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};

impl<'exec, E> Decode<'exec, Postgres<E>> for IpAddr
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut PostgresDecodeWrapper<'exec, '_>) -> Result<Self, E> {
    Ok(match dw.bytes() {
      [2, ..] => IpAddr::V4(Ipv4Addr::decode(dw)?),
      [3, ..] => IpAddr::V6(Ipv6Addr::decode(dw)?),
      _ => return Err(E::from(PostgresError::InvalidIpFormat.into())),
    })
  }
}
impl<E> Encode<Postgres<E>> for IpAddr
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut PostgresEncodeWrapper<'_>) -> Result<(), E> {
    match self {
      IpAddr::V4(ipv4_addr) => ipv4_addr.encode(ew),
      IpAddr::V6(ipv6_addr) => ipv6_addr.encode(ew),
    }
  }
}
impl<E> Typed<Postgres<E>> for IpAddr
where
  E: From<crate::Error>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    <Self as Typed<Postgres<E>>>::static_ty()
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    Some(Ty::Inet)
  }
}
test!(ipaddr_v4, IpAddr, IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)));
test!(ipaddr_v6, IpAddr, IpAddr::V6(Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8)));

impl<'exec, E> Decode<'exec, Postgres<E>> for Ipv4Addr
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut PostgresDecodeWrapper<'exec, '_>) -> Result<Self, E> {
    let [2, 32, 0, 4, b0, b1, b2, b3] = dw.bytes() else {
      return Err(E::from(PostgresError::InvalidIpFormat.into()));
    };
    Ok(Ipv4Addr::from([*b0, *b1, *b2, *b3]))
  }
}
impl<E> Encode<Postgres<E>> for Ipv4Addr
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut PostgresEncodeWrapper<'_>) -> Result<(), E> {
    let _ = ew.buffer().extend_from_copyable_slices([&[2, 32, 0, 4][..], &self.octets()])?;
    Ok(())
  }
}
impl<E> Typed<Postgres<E>> for Ipv4Addr
where
  E: From<crate::Error>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    <Self as Typed<Postgres<E>>>::static_ty()
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    Some(Ty::Inet)
  }
}
test!(ipv4, Ipv4Addr, Ipv4Addr::new(1, 2, 3, 4));

impl<'exec, E> Decode<'exec, Postgres<E>> for Ipv6Addr
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut PostgresDecodeWrapper<'exec, '_>) -> Result<Self, E> {
    let [3, 128, 0, 16, b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15] =
      dw.bytes()
    else {
      return Err(E::from(PostgresError::InvalidIpFormat.into()));
    };
    Ok(Ipv6Addr::from([
      *b0, *b1, *b2, *b3, *b4, *b5, *b6, *b7, *b8, *b9, *b10, *b11, *b12, *b13, *b14, *b15,
    ]))
  }
}
impl<E> Encode<Postgres<E>> for Ipv6Addr
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut PostgresEncodeWrapper<'_>) -> Result<(), E> {
    let _ = ew.buffer().extend_from_copyable_slices([&[3, 128, 0, 16][..], &self.octets()])?;
    Ok(())
  }
}
impl<E> Typed<Postgres<E>> for Ipv6Addr
where
  E: From<crate::Error>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    <Self as Typed<Postgres<E>>>::static_ty()
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    Some(Ty::Inet)
  }
}
test!(ipv6, Ipv6Addr, Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8));
