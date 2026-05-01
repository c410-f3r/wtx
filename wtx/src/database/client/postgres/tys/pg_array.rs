use crate::{
  codec::{Decode, Encode},
  collection::{ArrayVector, ArrayVectorU8, LinearStorageLen, TryExtend, Vector},
  database::{
    Typed,
    client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, PostgresError, Ty},
  },
  misc::{
    Lease, SingleTypeStorage, Usize,
    counter_writer::{CounterWriterBytesTy, i32_write},
  },
};
use alloc::vec::Vec;

/// Any 1-dimension collection of contiguous elements.
struct PgArray<T>(
  /// Array
  T,
);

impl<'de, E, T> Decode<'de, Postgres<E>> for PgArray<T>
where
  E: From<crate::Error>,
  T: Default + SingleTypeStorage + TryExtend<[T::Item; 1]>,
  T::Item: Decode<'de, Postgres<E>>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, '_>) -> Result<Self, E> {
    let [0, 0, 0, 1, 0, 0, 0, 0, a, b, c, d, e, f, g, h, 0, 0, 0, 1, rest @ ..] = dw.bytes() else {
      return Err(crate::Error::from(PostgresError::InvalidArray).into());
    };
    *dw.ty_mut() = Ty::from_arbitrary_u32(u32::from_be_bytes([*a, *b, *c, *d]));
    let len = u32::from_be_bytes([*e, *f, *g, *h]);
    let mut bytes = rest;
    let mut rslt = T::default();
    for _ in 0..len {
      let [i, j, k, l, local_rest0 @ ..] = bytes else {
        return Err(crate::Error::from(PostgresError::InvalidArray).into());
      };
      let len = Usize::from(u32::from_be_bytes([*i, *j, *k, *l])).usize();
      let Some((elem_bytes, local_rest1)) = local_rest0.split_at_checked(len) else {
        return Err(crate::Error::from(PostgresError::InvalidArray).into());
      };
      *dw.bytes_mut() = elem_bytes;
      rslt.try_extend([T::Item::decode(dw)?])?;
      bytes = local_rest1;
    }
    Ok(Self(rslt))
  }
}

impl<E, T> Encode<Postgres<E>> for PgArray<T>
where
  E: From<crate::Error>,
  T: SingleTypeStorage + Lease<[T::Item]>,
  T::Item: Encode<Postgres<E>> + Typed<Postgres<E>>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    let slice = self.0.lease();
    ew.buffer().extend_from_slices([
      &[0, 0, 0, 1, 0, 0, 0, 0][..],
      &u32::from(T::Item::static_ty().unwrap_or(Ty::Custom(0))).to_be_bytes(),
      &u32::try_from(slice.len()).map_err(crate::Error::from)?.to_be_bytes(),
      &[0, 0, 0, 1][..],
    ])?;
    for elem in slice {
      if elem.is_null() {
        ew.buffer().extend_from_slice(&(-1i32).to_be_bytes())?;
      } else {
        i32_write(CounterWriterBytesTy::IgnoresLen, None, ew.buffer(), |local_sw| {
          elem.encode(&mut EncodeWrapper::new(local_sw))
        })?;
      }
    }
    Ok(())
  }
}

impl<'de, E, T, const N: usize> Decode<'de, Postgres<E>> for [T; N]
where
  E: From<crate::Error>,
  T: Decode<'de, Postgres<E>>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, '_>) -> Result<Self, E> {
    Ok(ArrayVectorU8::<T, N>::decode(dw)?.into_inner()?)
  }
}

impl<'de, E, L, T, const N: usize> Decode<'de, Postgres<E>> for ArrayVector<L, T, N>
where
  E: From<crate::Error>,
  L: LinearStorageLen,
  T: Decode<'de, Postgres<E>>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, '_>) -> Result<Self, E> {
    Ok(PgArray::<ArrayVector<L, T, N>>::decode(dw)?.0)
  }
}

impl<'de, E, T> Decode<'de, Postgres<E>> for Vec<T>
where
  E: From<crate::Error>,
  T: Decode<'de, Postgres<E>>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, '_>) -> Result<Self, E> {
    Ok(PgArray::<Vec<T>>::decode(dw)?.0)
  }
}

impl<'de, E, T> Decode<'de, Postgres<E>> for Vector<T>
where
  E: From<crate::Error>,
  T: Decode<'de, Postgres<E>>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, '_>) -> Result<Self, E> {
    Ok(PgArray::<Vector<T>>::decode(dw)?.0)
  }
}

impl<E, T, const N: usize> Encode<Postgres<E>> for [T; N]
where
  E: From<crate::Error>,
  T: Encode<Postgres<E>> + Typed<Postgres<E>>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    PgArray(self).encode(ew)
  }
}

impl<E, L, T, const N: usize> Encode<Postgres<E>> for ArrayVector<L, T, N>
where
  E: From<crate::Error>,
  L: LinearStorageLen,
  T: Encode<Postgres<E>> + Typed<Postgres<E>>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    PgArray(self).encode(ew)
  }
}

impl<E, T> Encode<Postgres<E>> for Vec<T>
where
  E: From<crate::Error>,
  T: Encode<Postgres<E>> + Typed<Postgres<E>>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    PgArray(self).encode(ew)
  }
}

impl<E, T> Encode<Postgres<E>> for Vector<T>
where
  E: From<crate::Error>,
  T: Encode<Postgres<E>> + Typed<Postgres<E>>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    PgArray(self).encode(ew)
  }
}

impl<E, T, const N: usize> Typed<Postgres<E>> for [T; N]
where
  E: From<crate::Error>,
  T: Typed<Postgres<E>>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    T::static_ty().and_then(|el| el.array_ty())
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    T::static_ty().and_then(|el| el.array_ty())
  }
}

impl<E, L, T, const N: usize> Typed<Postgres<E>> for ArrayVector<L, T, N>
where
  E: From<crate::Error>,
  L: LinearStorageLen,
  T: Typed<Postgres<E>>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    T::static_ty().and_then(|el| el.array_ty())
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    T::static_ty().and_then(|el| el.array_ty())
  }
}

impl<E, T> Typed<Postgres<E>> for Vec<T>
where
  E: From<crate::Error>,
  T: Typed<Postgres<E>>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    T::static_ty().and_then(|el| el.array_ty())
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    T::static_ty().and_then(|el| el.array_ty())
  }
}

impl<E, T> Typed<Postgres<E>> for Vector<T>
where
  E: From<crate::Error>,
  T: Typed<Postgres<E>>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    T::static_ty().and_then(|el| el.array_ty())
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    T::static_ty().and_then(|el| el.array_ty())
  }
}
