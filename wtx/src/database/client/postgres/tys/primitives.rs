use crate::{
  codec::{Decode, Encode},
  database::{
    DatabaseError, Typed,
    client::postgres::{Postgres, PostgresDecodeWrapper, PostgresEncodeWrapper, Ty},
  },
  misc::Usize,
};

// bool

impl<E> Decode<'_, Postgres<E>> for bool
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut PostgresDecodeWrapper<'_, '_>) -> Result<Self, E> {
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
  fn encode(&self, ew: &mut PostgresEncodeWrapper<'_>) -> Result<(), E> {
    ew.buffer().push((*self).into())?;
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

impl_primitive!(37.0, [b0, b1, b2, b3], Ty::Float4, f32);
impl_primitive!(37.0, [b0, b1, b2, b3, b4, b5, b6, b7], Ty::Float8, f64);

impl_primitive!(37, [b0], Ty::Char, i8);
impl_primitive!(37, [b0, b1], Ty::Int2, i16);
impl_primitive!(37, [b0, b1, b2, b3], Ty::Int4, i32);
impl_primitive!(37, [b0, b1, b2, b3, b4, b5, b6, b7], Ty::Int8, i64);

// A hack, more or less. `Ty::Char` is not used because of arrays. For example, `[u8; 3]` would
// imply `CHAR[]`
impl_primitive!(37, [ab0], Ty::Bytea, u8);
impl_primitive!(37, [b0, b1], Ty::Int2, u16, i16);
impl_primitive!(37, [b0, b1, b2, b3], Ty::Int4, u32, i32);
impl_primitive!(37, [b0, b1, b2, b3, b4, b5, b6, b7], Ty::Int8, u64, i64);
