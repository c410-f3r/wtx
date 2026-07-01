use crate::{
  codec::{Decode, Encode},
  collections::{ArrayString, LinearStorageLen},
  database::{
    Typed,
    client::postgres::{Postgres, PostgresDecodeWrapper, PostgresEncodeWrapper, Ty},
  },
  misc::from_utf8_basic,
};
use alloc::string::String;

// &[u8]

impl<'exec, E> Decode<'exec, Postgres<E>> for &'exec [u8]
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut PostgresDecodeWrapper<'exec, '_>) -> Result<Self, E> {
    Ok(dw.bytes())
  }
}
impl<E> Encode<Postgres<E>> for &[u8]
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut PostgresEncodeWrapper<'_>) -> Result<(), E> {
    ew.buffer().extend_from_copyable_slice(self)?;
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
    Some(Ty::Bytea)
  }
}
test!(bytes, &[u8], &[1u8, 2, 3, 4]);

// &mut [u8]

impl<E> Encode<Postgres<E>> for &mut [u8]
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut PostgresEncodeWrapper<'_>) -> Result<(), E> {
    ew.buffer().extend_from_copyable_slice(self)?;
    Ok(())
  }
}
impl<E> Typed<Postgres<E>> for &mut [u8]
where
  E: From<crate::Error>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    Some(Ty::Bytea)
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    Some(Ty::Bytea)
  }
}

// str

impl<E> Encode<Postgres<E>> for str
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut PostgresEncodeWrapper<'_>) -> Result<(), E> {
    ew.buffer().extend_from_copyable_slice(self.as_bytes())?;
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
  fn decode(dw: &mut PostgresDecodeWrapper<'exec, '_>) -> Result<Self, E> {
    Ok(from_utf8_basic(dw.bytes()).map_err(crate::Error::from)?)
  }
}
impl<E> Encode<Postgres<E>> for &str
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut PostgresEncodeWrapper<'_>) -> Result<(), E> {
    ew.buffer().extend_from_copyable_slice(self.as_bytes())?;
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

// ArrayString

impl<E, L, const N: usize> Decode<'_, Postgres<E>> for ArrayString<L, N>
where
  E: From<crate::Error>,
  L: LinearStorageLen,
{
  #[inline]
  fn decode(dw: &mut PostgresDecodeWrapper<'_, '_>) -> Result<Self, E> {
    Ok(from_utf8_basic(dw.bytes()).map_err(Into::into)?.try_into()?)
  }
}
impl<E, L, const N: usize> Encode<Postgres<E>> for ArrayString<L, N>
where
  E: From<crate::Error>,
  L: LinearStorageLen,
{
  #[inline]
  fn encode(&self, ew: &mut PostgresEncodeWrapper<'_>) -> Result<(), E> {
    ew.buffer().extend_from_copyable_slice(self.as_bytes())?;
    Ok(())
  }
}
impl<E, L, const N: usize> Typed<Postgres<E>> for ArrayString<L, N>
where
  E: From<crate::Error>,
  L: LinearStorageLen,
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
test!(array_string, crate::collections::ArrayStringU8<4>, ArrayString::try_from("123").unwrap());

// String

impl<E> Decode<'_, Postgres<E>> for String
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut PostgresDecodeWrapper<'_, '_>) -> Result<Self, E> {
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
  fn encode(&self, ew: &mut PostgresEncodeWrapper<'_>) -> Result<(), E> {
    ew.buffer().extend_from_copyable_slice(self.as_bytes())?;
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
