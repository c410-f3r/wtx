use crate::{
  database::{
    DatabaseError, Typed,
    client::mysql::{DecodeWrapper, EncodeWrapper, Mysql, Ty, TyParams},
  },
  de::{Decode, Encode},
  misc::Usize,
};

impl<E> Decode<'_, Mysql<E>> for ()
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), _: &mut DecodeWrapper<'_, '_>) -> Result<Self, E> {
    Ok(())
  }
}

// bool

impl<E> Decode<'_, Mysql<E>> for bool
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dw: &mut DecodeWrapper<'_, '_>) -> Result<Self, E> {
    let &[byte] = dw.bytes() else {
      return Err(E::from(
        DatabaseError::UnexpectedBufferSize {
          expected: 1,
          received: Usize::from(dw.bytes().len()).into_saturating_u32(),
        }
        .into(),
      ));
    };
    Ok(byte != 0)
  }
}
impl<E> Encode<Mysql<E>> for bool
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
    ew.buffer().push((*self).into())?;
    Ok(())
  }
}
impl<E> Typed<Mysql<E>> for bool
where
  E: From<crate::Error>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<TyParams> {
    <Self as Typed<Mysql<E>>>::static_ty()
  }

  #[inline]
  fn static_ty() -> Option<TyParams> {
    Some(TyParams::unsigned(Ty::Tiny))
  }
}

macro_rules! impl_integer_from_array {
      ($instance:expr, [$($elem:ident),+], ($signed:ident, $signed_pg_ty:expr), ($unsigned:ident, $unsigned_pg_ty:expr)) => {
        impl_primitive_from_array!($instance, [$($elem),+], $signed, $signed_pg_ty);

        impl<E> Decode<'_, Mysql<E>> for $unsigned
        where
          E: From<crate::Error>,
        {
          #[inline]
          fn decode(_: &mut (), dw: &mut DecodeWrapper<'_, '_>) -> Result<Self, E> {
            if let &[$($elem,)+] = dw.bytes() {
              return Ok(<Self>::from_le_bytes([$($elem),+]));
            }
            Err(E::from(DatabaseError::UnexpectedBufferSize {
              expected: Usize::from(size_of::<Self>()).into_saturating_u32(),
              received: Usize::from(dw.bytes().len()).into_saturating_u32()
            }.into()))
          }
        }
        impl<E> Encode<Mysql<E>> for $unsigned
        where
          E: From<crate::Error>,
        {
          #[inline]
          fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
            ew.buffer().extend_from_copyable_slice(&self.to_le_bytes()).map_err(Into::into)?;
            Ok(())
          }
        }
        impl<E> Typed<Mysql<E>> for $unsigned
        where
          E: From<crate::Error>
        {
          #[inline]
          fn runtime_ty(&self) -> Option<TyParams> {
            <Self as Typed<Mysql<E>>>::static_ty()
          }

          #[inline]
          fn static_ty() -> Option<TyParams> {
            Some($unsigned_pg_ty)
          }
        }

        test!($unsigned, $unsigned, $instance);
      };
    }

macro_rules! impl_primitive_from_array {
      ($instance:expr, [$($elem:ident),+], $ty:ident, $pg_ty:expr) => {
        impl<E> Decode<'_, Mysql<E>> for $ty
        where
          E: From<crate::Error>,
        {
          #[inline]
          fn decode(_: &mut (), dw: &mut DecodeWrapper<'_, '_>) -> Result<Self, E> {
            if let &[$($elem,)+] = dw.bytes() {
              return Ok(<Self>::from_le_bytes([$($elem),+]));
            }
            Err(E::from(DatabaseError::UnexpectedBufferSize {
              expected: Usize::from(size_of::<Self>()).into_saturating_u32(),
              received: Usize::from(dw.bytes().len()).into_saturating_u32()
            }.into()))
          }
        }

        impl<E> Encode<Mysql<E>> for $ty
        where
          E: From<crate::Error>,
        {
          #[inline]
          fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
            ew.buffer().extend_from_copyable_slice(&self.to_le_bytes()).map_err(Into::into)?;
            Ok(())
          }
        }

        impl<E> Typed<Mysql<E>> for $ty
        where
          E: From<crate::Error>
        {
          #[inline]
          fn runtime_ty(&self) -> Option<TyParams> {
            <Self as Typed<Mysql<E>>>::static_ty()
          }

          #[inline]
          fn static_ty() -> Option<TyParams> {
            Some($pg_ty)
          }
        }

        test!($ty, $ty, $instance);
      }
    }

impl_integer_from_array!(
  37,
  [a],
  (i8, TyParams::binary(Ty::Tiny)),
  (u8, TyParams::unsigned(Ty::Tiny))
);
impl_integer_from_array!(
  37,
  [a, b],
  (i16, TyParams::binary(Ty::Short)),
  (u16, TyParams::unsigned(Ty::Short))
);
impl_integer_from_array!(
  37,
  [a, b, c, d],
  (i32, TyParams::binary(Ty::Long)),
  (u32, TyParams::unsigned(Ty::Long))
);
impl_integer_from_array!(
  37,
  [a, b, c, d, e, f, g, h],
  (i64, TyParams::binary(Ty::LongLong)),
  (u64, TyParams::unsigned(Ty::LongLong))
);

impl_primitive_from_array!(37.0, [a, b, c, d], f32, TyParams::binary(Ty::Float));
impl_primitive_from_array!(37.0, [a, b, c, d, e, f, g, h], f64, TyParams::binary(Ty::Double));
