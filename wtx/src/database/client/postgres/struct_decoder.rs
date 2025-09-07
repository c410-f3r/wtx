use crate::{
  database::client::postgres::{DecodeWrapper, Postgres, PostgresError, Ty},
  de::Decode,
  misc::Usize,
};
use core::marker::PhantomData;

/// Decodes a custom PostgreSQL type that represents a table into a Rust struct.
#[derive(Debug)]
pub struct StructDecoder<'de, E> {
  bytes: &'de [u8],
  phantom: PhantomData<fn() -> E>,
}

impl<'de, E> StructDecoder<'de, E>
where
  E: From<crate::Error>,
{
  /// Decodes initial data.
  #[inline]
  pub fn new(dw: &mut DecodeWrapper<'de>) -> Self {
    let bytes = if let [_, _, _, _, rest @ ..] = dw.bytes() { rest } else { dw.bytes() };
    Self { bytes, phantom: PhantomData }
  }

  /// Decodes a "non-null" element. Calls to this method must match the order in which the struct
  /// fields were encoded.
  pub fn decode<T>(&mut self) -> Result<T, E>
  where
    T: Decode<'de, Postgres<E>>,
  {
    Ok(self.decode_opt()?.ok_or_else(|| PostgresError::DecodingError.into())?)
  }

  /// Decodes a nullable element. Calls to this method must match the order in which the struct
  /// fields were encoded.
  pub fn decode_opt<T>(&mut self) -> Result<Option<T>, E>
  where
    T: Decode<'de, Postgres<E>>,
  {
    let [a, b, c, d, e, f, g, h, rest @ ..] = self.bytes else {
      return Ok(None);
    };
    let ty = Ty::from_arbitrary_u32(u32::from_be_bytes([*a, *b, *c, *d]));
    let Ok(length) = u32::try_from(i32::from_be_bytes([*e, *f, *g, *h])) else {
      self.bytes = rest;
      return Ok(None);
    };
    let Some((before, after)) = rest.split_at_checked(*Usize::from(length)) else {
      self.bytes = rest;
      return Ok(None);
    };
    self.bytes = after;
    Ok(Some(T::decode(&mut (), &mut DecodeWrapper::new(before, ty))?))
  }
}
