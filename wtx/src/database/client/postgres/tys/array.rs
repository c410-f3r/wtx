use crate::{
  collection::{ArrayString, ArrayVector, LinearStorageLen},
  database::{
    Typed,
    client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, Ty},
  },
  de::{Decode, Encode},
  misc::from_utf8_basic,
};

// ArrayString

impl<E, L, const N: usize> Decode<'_, Postgres<E>> for ArrayString<L, N>
where
  E: From<crate::Error>,
  L: LinearStorageLen,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'_, '_>) -> Result<Self, E> {
    Ok(from_utf8_basic(dw.bytes()).map_err(Into::into)?.try_into()?)
  }
}
impl<E, L, const N: usize> Encode<Postgres<E>> for ArrayString<L, N>
where
  E: From<crate::Error>,
  L: LinearStorageLen,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    ew.buffer().extend_from_slice(self.as_bytes())?;
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
test!(array_string, crate::collection::ArrayStringU8<4>, ArrayString::try_from("123").unwrap());

// ArrayVector

impl<E, L, const N: usize> Decode<'_, Postgres<E>> for ArrayVector<L, u8, N>
where
  E: From<crate::Error>,
  L: LinearStorageLen,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'_, '_>) -> Result<Self, E> {
    Ok(ArrayVector::try_from(dw.bytes())?)
  }
}
impl<E, L, const N: usize> Encode<Postgres<E>> for ArrayVector<L, u8, N>
where
  E: From<crate::Error>,
  L: LinearStorageLen,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    ew.buffer().extend_from_slice(self)?;
    Ok(())
  }
}
impl<E, L, const N: usize> Typed<Postgres<E>> for ArrayVector<L, u8, N>
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
    Some(Ty::ByteaArray)
  }
}
test!(array_vector, crate::collection::ArrayVectorU8<u8, 4>, ArrayVector::from_array([1, 2, 3, 4]));
