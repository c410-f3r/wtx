use crate::{
  database::{
    Typed,
    client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, PostgresError, Ty},
  },
  de::{Decode, Encode},
};
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};

impl<'exec, E> Decode<'exec, Postgres<E>> for IpAddr
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(aux: &mut (), dw: &mut DecodeWrapper<'exec>) -> Result<Self, E> {
    Ok(match dw.bytes() {
      [2, ..] => IpAddr::V4(Ipv4Addr::decode(aux, dw)?),
      [3, ..] => IpAddr::V6(Ipv6Addr::decode(aux, dw)?),
      _ => return Err(E::from(PostgresError::InvalidIpFormat.into())),
    })
  }
}
impl<E> Encode<Postgres<E>> for IpAddr
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, aux: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    match self {
      IpAddr::V4(ipv4_addr) => ipv4_addr.encode(aux, ew),
      IpAddr::V6(ipv6_addr) => ipv6_addr.encode(aux, ew),
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
  fn decode(_: &mut (), dw: &mut DecodeWrapper<'exec>) -> Result<Self, E> {
    let [2, 32, 0, 4, e, f, g, h] = dw.bytes() else {
      return Err(E::from(PostgresError::InvalidIpFormat.into()));
    };
    Ok(Ipv4Addr::from([*e, *f, *g, *h]))
  }
}
impl<E> Encode<Postgres<E>> for Ipv4Addr
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    ew.buffer().extend_from_slices([&[2, 32, 0, 4][..], &self.octets()])?;
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
  fn decode(_: &mut (), dw: &mut DecodeWrapper<'exec>) -> Result<Self, E> {
    let [3, 128, 0, 16, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t] = dw.bytes() else {
      return Err(E::from(PostgresError::InvalidIpFormat.into()));
    };
    Ok(Ipv6Addr::from([*e, *f, *g, *h, *i, *j, *k, *l, *m, *n, *o, *p, *q, *r, *s, *t]))
  }
}
impl<E> Encode<Postgres<E>> for Ipv6Addr
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    ew.buffer().extend_from_slices([&[3, 128, 0, 16][..], &self.octets()])?;
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
