use crate::{
  codec::{Decode, Encode},
  database::{
    DatabaseError, Typed,
    client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, Ty},
  },
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

impl_primitive!(37.0, [a, b, c, d], Ty::Float4, f32);
impl_primitive!(37.0, [a, b, c, d, e, f, g, h], Ty::Float8, f64);

impl_primitive!(37, [a], Ty::Char, i8);
impl_primitive!(37, [a, b], Ty::Int2, i16);
impl_primitive!(37, [a, b, c, d], Ty::Int4, i32);
impl_primitive!(37, [a, b, c, d, e, f, g, h], Ty::Int8, i64);

impl_primitive!(37, [a], Ty::Bytea, u8, i8);
impl_primitive!(37, [a, b], Ty::Int2, u16, i16);
impl_primitive!(37, [a, b, c, d], Ty::Int4, u32, i32);
impl_primitive!(37, [a, b, c, d, e, f, g, h], Ty::Int8, u64, i64);
