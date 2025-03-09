use crate::{
  database::{
    Typed,
    client::postgres::{EncodeWrapper, Postgres, Ty},
  },
  misc::Encode,
};
use core::marker::PhantomData;

/// Encodes a Rust struct into a custom PostgreSQL type that represents a table.
#[derive(Debug)]
pub struct StructEncoder<'buffer, 'ew, 'tmp, E> {
  ew: &'ew mut EncodeWrapper<'buffer, 'tmp>,
  len: u32,
  phantom: PhantomData<fn() -> E>,
  start: usize,
}

impl<'buffer, 'ew, 'tmp, E> StructEncoder<'buffer, 'ew, 'tmp, E>
where
  E: From<crate::Error>,
{
  /// Pushes initial encoding data.
  #[inline]
  pub fn new(ew: &'ew mut EncodeWrapper<'buffer, 'tmp>) -> Result<Self, E> {
    let start = ew.buffer()._len();
    ew.buffer().extend_from_slice(&[0; 4])?;
    Ok(Self { ew, len: 0, phantom: PhantomData, start })
  }

  /// Encodes `value` with the [`Ty`] originated from [`Typed`].
  #[inline]
  pub fn encode<T>(self, value: T) -> Result<Self, E>
  where
    T: Encode<Postgres<E>> + Typed<Postgres<E>>,
  {
    let ty = value.runtime_ty().unwrap_or(Ty::Any);
    self.encode_with_ty(value, ty)
  }

  /// Encodes `value` with the provided `ty`.
  #[inline]
  pub fn encode_with_ty<T>(mut self, value: T, ty: Ty) -> Result<Self, E>
  where
    T: Encode<Postgres<E>>,
  {
    self.ew.buffer().extend_from_slice(&u32::from(ty).to_be_bytes())?;
    if value.is_null() {
      self.ew.buffer().extend_from_slice(&(-1i32).to_be_bytes())?;
    } else {
      let len_start = self.ew.buffer()._len();
      self.ew.buffer().extend_from_slice(&[0; 4])?;
      let elem_start = self.ew.buffer()._len();
      value.encode(&mut (), self.ew)?;
      let len = self.ew.buffer()._len().wrapping_sub(elem_start).try_into().unwrap_or_default();
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

#[inline]
fn write_len(ew: &mut EncodeWrapper<'_, '_>, start: usize, len: u32) {
  let Some([a, b, c, d, ..]) = ew.buffer()._curr_bytes_mut().get_mut(start..) else {
    return;
  };
  let [e, f, g, h] = len.to_be_bytes();
  *a = e;
  *b = f;
  *c = g;
  *d = h;
}
