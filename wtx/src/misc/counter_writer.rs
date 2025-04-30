#![allow(dead_code, reason = "feature selection")]

use crate::{
  collection::Vector,
  misc::{LeaseMut, SuffixWriter},
};
use core::marker::PhantomData;

pub(crate) trait CounterWriter<E, V>
where
  E: From<crate::Error>,
  V: LeaseMut<Vector<u8>>,
{
  fn write(
    &self,
    sw: &mut SuffixWriter<V>,
    include_len: bool,
    prefix: Option<u8>,
    cb: impl FnOnce(&mut SuffixWriter<V>) -> Result<(), E>,
  ) -> Result<(), E>;

  fn write_iter<T>(
    &self,
    sw: &mut SuffixWriter<V>,
    iter: impl IntoIterator<Item = T>,
    prefix: Option<u8>,
    cb: impl FnMut(T, &mut SuffixWriter<V>) -> Result<(), E>,
  ) -> Result<(), E>;
}

#[inline]
fn write<E, V, const N: usize>(
  sw: &mut SuffixWriter<V>,
  include_len: bool,
  prefix: Option<u8>,
  sw_cb: impl FnOnce(&mut SuffixWriter<V>) -> Result<(), E>,
  value_cb: impl FnOnce(&mut SuffixWriter<V>, usize) -> Result<[u8; N], E>,
) -> Result<(), E>
where
  E: From<crate::Error>,
  V: LeaseMut<Vector<u8>>,
{
  let after_prefix = write_init::<_, _, N>(sw, prefix)?;
  let len_before = if include_len { after_prefix } else { sw._len() };
  sw_cb(sw)?;
  let value = value_cb(sw, len_before)?;
  write_prefix(after_prefix, sw, value);
  Ok(())
}

#[inline]
fn write_init<E, V, const N: usize>(
  sw: &mut SuffixWriter<V>,
  prefix: Option<u8>,
) -> Result<usize, E>
where
  E: From<crate::Error>,
  V: LeaseMut<Vector<u8>>,
{
  if let Some(elem) = prefix {
    sw._extend_from_byte(elem)?;
  }
  let after_prefix = sw._len();
  sw.extend_from_slice(&[0; N])?;
  Ok(after_prefix)
}

#[inline]
fn write_iter<E, T, V, const N: usize>(
  sw: &mut SuffixWriter<V>,
  iter: impl IntoIterator<Item = T>,
  prefix: Option<u8>,
  mut sw_cb: impl FnMut(T, &mut SuffixWriter<V>) -> Result<(), E>,
  value_cb: impl FnOnce(usize) -> Result<[u8; N], E>,
) -> Result<(), E>
where
  E: From<crate::Error>,
  V: LeaseMut<Vector<u8>>,
{
  let start = write_init::<_, _, N>(sw, prefix)?;
  let mut counter: usize = 0;
  for elem in iter.into_iter().take(u16::MAX.into()) {
    sw_cb(elem, sw)?;
    counter = counter.wrapping_add(1);
  }
  let value = value_cb(counter)?;
  write_prefix(start, sw, value);
  Ok(())
}

#[inline]
fn write_prefix<V, const N: usize>(start: usize, sw: &mut SuffixWriter<V>, value: [u8; N])
where
  V: LeaseMut<Vector<u8>>,
{
  let end = start.wrapping_add(value.len());
  if let Some(elem) = sw._curr_bytes_mut().get_mut(start..end) {
    elem.copy_from_slice(&value);
  }
}

macro_rules! impl_trait {
  (
    $name:ident,
    $ty:ident
  ) => {
    #[derive(Debug)]
    pub(crate) struct $name<E, V>(PhantomData<(E, V)>);

    impl<E, V> CounterWriter<E, V> for $name<E, V>
    where
      E: From<crate::Error>,
      V: LeaseMut<Vector<u8>>,
    {
      #[inline]
      fn write(
        &self,
        sw: &mut SuffixWriter<V>,
        include_len: bool,
        prefix: Option<u8>,
        cb: impl FnOnce(&mut SuffixWriter<V>) -> Result<(), E>,
      ) -> Result<(), E> {
        write(sw, include_len, prefix, cb, |local_sw, len_before| {
          let diff = local_sw._len().wrapping_sub(len_before);
          Ok($ty::try_from(diff).map_err(Into::into)?.to_be_bytes())
        })
      }

      #[inline]
      fn write_iter<T>(
        &self,
        sw: &mut SuffixWriter<V>,
        iter: impl IntoIterator<Item = T>,
        prefix: Option<u8>,
        cb: impl FnMut(T, &mut SuffixWriter<V>) -> Result<(), E>,
      ) -> Result<(), E> {
        write_iter(sw, iter, prefix, cb, |counter| {
          Ok($ty::try_from(counter).map_err(Into::into)?.to_be_bytes())
        })
      }
    }

    impl<E, V> Default for $name<E, V> {
      #[inline]
      fn default() -> Self {
        Self(PhantomData)
      }
    }
  };
}

impl_trait!(I16Counter, i16);
impl_trait!(U16Counter, u16);
impl_trait!(I32Counter, i32);
