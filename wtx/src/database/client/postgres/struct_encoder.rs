use crate::{
  codec::Encode,
  database::{
    Typed,
    client::postgres::{Postgres, PostgresEncodeWrapper, Ty},
  },
};
use core::marker::PhantomData;

/// Encodes a Rust struct into a custom PostgreSQL type that represents a table.
#[derive(Debug)]
pub struct StructEncoder<'inner, 'ew, 'outer, E> {
  ew: &'ew mut PostgresEncodeWrapper<'inner, 'outer>,
  len: u32,
  phantom: PhantomData<fn() -> E>,
  start: usize,
}

impl<'inner, 'ew, 'outer, E> StructEncoder<'inner, 'ew, 'outer, E>
where
  E: From<crate::Error>,
{
  /// Pushes initial encoding data.
  #[inline]
  pub fn new(ew: &'ew mut PostgresEncodeWrapper<'inner, 'outer>) -> Result<Self, E> {
    let start = ew.buffer().inner().len();
    ew.buffer().inner_mut().extend_from_copyable_slice(&[0; 4])?;
    Ok(Self { ew, len: 0, phantom: PhantomData, start })
  }

  /// Encodes `value` with the [`Ty`] originated from [`Typed`].
  #[inline]
  pub fn encode<T>(self, value: T) -> Result<Self, E>
  where
    T: Encode<Postgres<E>> + Typed<Postgres<E>>,
  {
    let ty = value.runtime_ty().unwrap_or(Ty::Custom(0));
    self.encode_with_ty(value, ty)
  }

  /// Encodes `value` with the provided `ty`.
  #[inline]
  pub fn encode_with_ty<T>(mut self, value: T, ty: Ty) -> Result<Self, E>
  where
    T: Encode<Postgres<E>>,
  {
    let buffer = self.ew.buffer().inner_mut();
    buffer.extend_from_copyable_slice(&u32::from(ty).to_be_bytes())?;
    if value.is_null() {
      buffer.extend_from_copyable_slice(&(-1i32).to_be_bytes())?;
    } else {
      let len_start = buffer.len();
      buffer.extend_from_copyable_slice(&[0; 4])?;
      let idx = buffer.len();
      value.encode(self.ew)?;
      let len = self.ew.buffer().inner().len().wrapping_sub(idx).try_into().unwrap_or_default();
      write_len(self.ew, len_start, len);
    }
    self.len = self.len.wrapping_add(1);
    Ok(self)
  }
}

impl<E> Drop for StructEncoder<'_, '_, '_, E> {
  #[inline]
  fn drop(&mut self) {
    write_len(self.ew, self.start, self.len);
  }
}

fn write_len(ew: &mut PostgresEncodeWrapper<'_, '_>, start: usize, len: u32) {
  let Some([b0, b1, b2, b3, ..]) = ew.buffer().inner_mut().get_mut(start..) else {
    return;
  };
  let [b4, b5, b6, b7] = len.to_be_bytes();
  *b0 = b4;
  *b1 = b5;
  *b2 = b6;
  *b3 = b7;
}
