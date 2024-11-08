use crate::database::{
  client::postgres::{EncodeValue, Postgres, Ty},
  Encode, Typed,
};
use core::marker::PhantomData;

/// Encodes a Rust struct into a custom PostgreSQL type that represents a table.
#[derive(Debug)]
pub struct StructEncoder<'buffer, 'ev, 'tmp, E> {
  ev: &'ev mut EncodeValue<'buffer, 'tmp>,
  len: u32,
  phantom: PhantomData<fn() -> E>,
  start: usize,
}

impl<'buffer, 'ev, 'tmp, E> StructEncoder<'buffer, 'ev, 'tmp, E>
where
  E: From<crate::Error>,
{
  /// Pushes initial encoding data.
  #[inline]
  pub fn new(ev: &'ev mut EncodeValue<'buffer, 'tmp>) -> Result<Self, E> {
    let start = ev.fbw()._len();
    ev.fbw()._extend_from_slice(&[0; 4]).map_err(Into::into)?;
    Ok(Self { ev, len: 0, phantom: PhantomData, start })
  }

  /// Encodes `value` with the [`Ty`] originated from [`Typed`].
  #[inline]
  pub fn encode<T>(self, value: T) -> Result<Self, E>
  where
    T: Encode<Postgres<E>> + Typed<Postgres<E>>,
  {
    self.encode_with_ty(value, T::TY)
  }

  /// Encodes `value` with the provided `ty`.
  #[inline]
  pub fn encode_with_ty<T>(mut self, value: T, ty: Ty) -> Result<Self, E>
  where
    T: Encode<Postgres<E>>,
  {
    self.ev.fbw()._extend_from_slice(&u32::from(ty).to_be_bytes()).map_err(Into::into)?;
    if value.is_null() {
      self.ev.fbw()._extend_from_slice(&(-1i32).to_be_bytes()).map_err(Into::into)?;
    } else {
      let len_start = self.ev.fbw()._len();
      self.ev.fbw()._extend_from_slice(&[0; 4]).map_err(Into::into)?;
      let elem_start = self.ev.fbw()._len();
      value.encode(self.ev)?;
      let len = self.ev.fbw()._len().wrapping_sub(elem_start).try_into().unwrap_or_default();
      write_len(self.ev, len_start, len);
    }
    self.len = self.len.wrapping_add(1);
    Ok(self)
  }
}

impl<'buffer, 'fbw, 'vec, E> Drop for StructEncoder<'buffer, 'fbw, 'vec, E> {
  #[inline]
  fn drop(&mut self) {
    write_len(self.ev, self.start, self.len);
  }
}

#[inline]
fn write_len(ev: &mut EncodeValue<'_, '_>, start: usize, len: u32) {
  let Some([a, b, c, d, ..]) = ev.fbw()._curr_bytes_mut().get_mut(start..) else {
    return;
  };
  let [e, f, g, h] = len.to_be_bytes();
  *a = e;
  *b = f;
  *c = g;
  *d = h;
}
