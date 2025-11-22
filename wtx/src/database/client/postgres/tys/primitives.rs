use crate::{
  database::{
    DatabaseError, Typed,
    client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, Ty},
  },
  de::{Decode, Encode},
  misc::Usize,
};

// bool

impl<E> Decode<'_, Postgres<E>> for bool
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'_, '_>) -> Result<Self, E> {
    let &[byte] = dw.bytes() else {
      return Err(E::from(
        DatabaseError::UnexpectedBufferSize {
          expected: 1,
          received: Usize::from(dw.bytes().len()).into_u64().try_into().unwrap_or(u32::MAX),
        }
        .into(),
      ));
    };
    Ok(byte != 0)
  }
}
impl<E> Encode<Postgres<E>> for bool
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    ew.buffer().extend_from_byte((*self).into())?;
    Ok(())
  }
}
impl<E> Typed<Postgres<E>> for bool
where
  E: From<crate::Error>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    <Self as Typed<Postgres<E>>>::static_ty()
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    Some(Ty::Bool)
  }
}

kani!(bool_true, bool);
kani!(bool_false, bool);

impl_primitive!(37.0, [a, b, c, d], f32, Ty::Float4);
impl_primitive!(37.0, [a, b, c, d, e, f, g, h], f64, Ty::Float8);

impl_primitive!(37, [a], i8, Ty::Char);
impl_primitive!(37, [a, b], i16, Ty::Int2);
impl_primitive!(37, [a, b, c, d], i32, Ty::Int4);
impl_primitive!(37, [a, b, c, d, e, f, g, h], i64, Ty::Int8);

impl_primitive!(37, [a], u8, Ty::Bytea);
impl_primitive!(37, [a, b], u16, Ty::ByteaArray);
impl_primitive!(37, [a, b, c, d], u32, Ty::ByteaArray);
impl_primitive!(37, [a, b, c, d, e, f, g, h], u64, Ty::ByteaArray);
