use crate::{
  database::{
    Typed,
    client::postgres::{Postgres, PostgresDecodeWrapper, PostgresEncodeWrapper, Ty},
  },
  de::{Decode, Encode},
  misc::from_utf8_basic,
};
use alloc::string::String;

// &[u8]

impl<'exec, E> Decode<'exec, Postgres<E>> for &'exec [u8]
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dw: &mut PostgresDecodeWrapper<'exec, '_>) -> Result<Self, E> {
    Ok(dw.bytes())
  }
}
impl<E> Encode<Postgres<E>> for &[u8]
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut PostgresEncodeWrapper<'_, '_>) -> Result<(), E> {
    ew.buffer().extend_from_slice(self)?;
    Ok(())
  }
}
impl<E> Typed<Postgres<E>> for &[u8]
where
  E: From<crate::Error>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    <Self as Typed<Postgres<E>>>::static_ty()
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    Some(Ty::ByteaArray)
  }
}
test!(bytes, &[u8], &[1, 2, 3, 4]);

// str

impl<E> Encode<Postgres<E>> for str
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut PostgresEncodeWrapper<'_, '_>) -> Result<(), E> {
    ew.buffer().extend_from_slice(self.as_bytes())?;
    Ok(())
  }
}
impl<E> Typed<Postgres<E>> for str
where
  E: From<crate::Error>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    <&Self as Typed<Postgres<E>>>::static_ty()
  }
}

// &str

impl<'exec, E> Decode<'exec, Postgres<E>> for &'exec str
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dw: &mut PostgresDecodeWrapper<'exec, '_>) -> Result<Self, E> {
    Ok(from_utf8_basic(dw.bytes()).map_err(crate::Error::from)?)
  }
}
impl<E> Encode<Postgres<E>> for &str
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut PostgresEncodeWrapper<'_, '_>) -> Result<(), E> {
    ew.buffer().extend_from_slice(self.as_bytes())?;
    Ok(())
  }
}
impl<E> Typed<Postgres<E>> for &str
where
  E: From<crate::Error>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    <Self as Typed<Postgres<E>>>::static_ty()
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    Some(Ty::Text)
  }
}
test!(str, &str, "1234");

// String

impl<E> Decode<'_, Postgres<E>> for String
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dw: &mut PostgresDecodeWrapper<'_, '_>) -> Result<Self, E> {
    match from_utf8_basic(dw.bytes()).map_err(crate::Error::from) {
      Ok(elem) => Ok(elem.into()),
      Err(err) => Err(err.into()),
    }
  }
}
impl<E> Encode<Postgres<E>> for String
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut PostgresEncodeWrapper<'_, '_>) -> Result<(), E> {
    ew.buffer().extend_from_slice(self.as_bytes())?;
    Ok(())
  }
}
impl<E> Typed<Postgres<E>> for String
where
  E: From<crate::Error>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    <Self as Typed<Postgres<E>>>::static_ty()
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    Some(Ty::Text)
  }
}
kani!(string, String);
