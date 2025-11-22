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
  fn decode(_: &mut DecodeWrapper<'_, '_>) -> Result<Self, E> {
    Ok(())
  }
}

// bool

impl<E> Decode<'_, Mysql<E>> for bool
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'_, '_>) -> Result<Self, E> {
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
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
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

impl_primitive!(37.0, [a, b, c, d], f32, TyParams::binary(Ty::Float));
impl_primitive!(37.0, [a, b, c, d, e, f, g, h], f64, TyParams::binary(Ty::Double));

impl_primitive!(37, [a], i8, TyParams::binary(Ty::Tiny));
impl_primitive!(37, [a, b], i16, TyParams::binary(Ty::Short));
impl_primitive!(37, [a, b, c, d], i32, TyParams::binary(Ty::Long));
impl_primitive!(37, [a, b, c, d, e, f, g, h], i64, TyParams::binary(Ty::LongLong));

impl_primitive!(37, [a], u8, TyParams::unsigned(Ty::Tiny));
impl_primitive!(37, [a, b], u16, TyParams::unsigned(Ty::Short));
impl_primitive!(37, [a, b, c, d], u32, TyParams::unsigned(Ty::Long));
impl_primitive!(37, [a, b, c, d, e, f, g, h], u64, TyParams::unsigned(Ty::LongLong));
