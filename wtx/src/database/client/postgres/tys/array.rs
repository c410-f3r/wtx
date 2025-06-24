use crate::{
  collection::{ArrayString, ArrayVector},
  database::{
    Typed,
    client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, Ty},
  },
  de::{Decode, Encode},
  misc::from_utf8_basic,
};

// ArrayString

impl<E, const N: usize> Decode<'_, Postgres<E>> for ArrayString<N>
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
    Ok(from_utf8_basic(dw.bytes()).map_err(Into::into)?.try_into()?)
  }
}
impl<E, const N: usize> Encode<Postgres<E>> for ArrayString<N>
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    ew.buffer().extend_from_slice(self.as_str().as_bytes())?;
    Ok(())
  }
}
impl<E, const N: usize> Typed<Postgres<E>> for ArrayString<N>
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
test!(array_string, ArrayString<4>, ArrayString::try_from("123").unwrap());

// ArrayVector

impl<E, const N: usize> Decode<'_, Postgres<E>> for ArrayVector<u8, N>
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
    Ok(ArrayVector::try_from(dw.bytes())?)
  }
}
impl<E, const N: usize> Encode<Postgres<E>> for ArrayVector<u8, N>
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    ew.buffer().extend_from_slice(self)?;
    Ok(())
  }
}
impl<E, const N: usize> Typed<Postgres<E>> for ArrayVector<u8, N>
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
test!(array_vector, ArrayVector<u8, 4>, ArrayVector::from_array([1, 2, 3, 4]));
