use crate::{
  database::{
    DatabaseError, Typed,
    client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, PostgresError, Ty},
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

macro_rules! impl_integer_from_array {
    ($instance:expr, [$($elem:ident),+], ($signed:ident, $signed_pg_ty:expr), ($unsigned:ident, $unsigned_pg_ty:expr)) => {
      impl_primitive_from_array!($instance, [$($elem),+], $signed, $signed_pg_ty);

      impl<E> Decode<'_, Postgres<E>> for $unsigned
      where
        E: From<crate::Error>,
      {
        #[inline]
        fn decode(dw: &mut DecodeWrapper<'_, '_>) -> Result<Self, E> {
          <$signed as Decode::<Postgres<E>>>::decode(dw)?
            .try_into()
            .map_err(|_err| E::from(PostgresError::InvalidPostgresUint.into()))
        }
      }
      impl<E> Encode<Postgres<E>> for $unsigned
      where
        E: From<crate::Error>,
      {
        #[inline]
        fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
          if *self > const { $unsigned::MAX >> 1 } {
            return Err(E::from(PostgresError::InvalidPostgresUint.into()));
          }
          ew.buffer().extend_from_slice(&self.to_be_bytes())?;
          Ok(())
        }
      }
      impl<E> Typed<Postgres<E>> for $unsigned
      where
        E: From<crate::Error>
      {
        #[inline]
        fn runtime_ty(&self) -> Option<Ty> {
          <Self as Typed<Postgres<E>>>::static_ty()
        }

        #[inline]
        fn static_ty() -> Option<Ty> {
          Some($unsigned_pg_ty)
        }
      }

      test!($unsigned, $unsigned, $instance);
    };
  }

macro_rules! impl_primitive_from_array {
    ($instance:expr, [$($elem:ident),+], $ty:ident, $pg_ty:expr) => {
      impl<E> Decode<'_, Postgres<E>> for $ty
      where
        E: From<crate::Error>,
      {
          #[inline]
          fn decode(dw: &mut DecodeWrapper<'_, '_>) -> Result<Self, E> {
          if let &[$($elem,)+] = dw.bytes() {
            return Ok(<Self>::from_be_bytes([$($elem),+]));
          }
          Err(E::from(DatabaseError::UnexpectedBufferSize {
            expected: Usize::from(size_of::<$ty>()).into_u64().try_into().unwrap_or(u32::MAX),
            received: Usize::from(dw.bytes().len()).into_u64().try_into().unwrap_or(u32::MAX)
          }.into()))
        }
      }

      impl<E> Encode<Postgres<E>> for $ty
      where
        E: From<crate::Error>,
      {
        #[inline]
        fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
          ew.buffer().extend_from_slice(&self.to_be_bytes())?;
          Ok(())
        }
      }

      impl<E> Typed<Postgres<E>> for $ty
      where
        E: From<crate::Error>
      {
        #[inline]
        fn runtime_ty(&self) -> Option<Ty> {
          <Self as Typed<Postgres<E>>>::static_ty()
        }

        #[inline]
        fn static_ty() -> Option<Ty> {
          Some($pg_ty)
        }
      }

      test!($ty, $ty, $instance);
    }
  }

impl_integer_from_array!(37, [a], (i8, Ty::Char), (u8, Ty::Bytea));
impl_integer_from_array!(37, [a, b], (i16, Ty::Int2), (u16, Ty::Int2));
impl_integer_from_array!(37, [a, b, c, d], (i32, Ty::Int4), (u32, Ty::Int4));
impl_integer_from_array!(37, [a, b, c, d, e, f, g, h], (i64, Ty::Int8), (u64, Ty::Int8));

impl_primitive_from_array!(37.0, [a, b, c, d], f32, Ty::Float4);
impl_primitive_from_array!(37.0, [a, b, c, d, e, f, g, h], f64, Ty::Float8);
