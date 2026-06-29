use crate::{
  collections::TryExtend,
  misc::{Lease, LeaseMut},
};

#[derive(Clone, Copy, Debug)]
pub(crate) enum CounterWriterIterTy {
  Bytes(CounterWriterBytesTy),
  #[cfg(feature = "postgres")]
  Elements,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum CounterWriterBytesTy {
  #[cfg(feature = "postgres")]
  IncludesLen,
  IgnoresLen,
}

#[inline]
fn write<E, T, const N: usize>(
  cwbt: CounterWriterBytesTy,
  prefix: Option<u8>,
  vec: &mut T,
  sw_cb: impl FnOnce(&mut T) -> Result<(), E>,
  value_cb: impl FnOnce(&mut T, usize) -> Result<[u8; N], E>,
) -> Result<(), E>
where
  E: From<crate::Error>,
  T: LeaseMut<[u8]> + for<'any> TryExtend<&'any [u8]>,
{
  let len_write_begin = writer_len_begin::<_, _, N>(prefix, vec)?;
  let len_begin = match cwbt {
    #[cfg(feature = "postgres")]
    CounterWriterBytesTy::IncludesLen => len_write_begin,
    CounterWriterBytesTy::IgnoresLen => vec.lease().len(),
  };
  sw_cb(vec)?;
  let value = value_cb(vec, len_begin)?;
  write_len_end(len_write_begin, vec, value);
  Ok(())
}

#[inline]
fn write_iter<E, T, U, const N: usize>(
  cwit: CounterWriterIterTy,
  iter: impl IntoIterator<Item = U>,
  prefix: Option<u8>,
  vec: &mut T,
  mut sw_cb: impl FnMut(U, &mut T) -> Result<(), E>,
  value_cb: impl FnOnce(usize, &mut T, usize) -> Result<[u8; N], E>,
) -> Result<(), E>
where
  E: From<crate::Error>,
  T: LeaseMut<[u8]> + for<'any> TryExtend<&'any [u8]>,
{
  let len_write_begin = writer_len_begin::<_, _, N>(prefix, vec)?;
  let len_begin = match cwit {
    #[cfg(feature = "postgres")]
    CounterWriterIterTy::Bytes(CounterWriterBytesTy::IncludesLen) => len_write_begin,
    CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen) => vec.lease().len(),
    #[cfg(feature = "postgres")]
    CounterWriterIterTy::Elements => 0,
  };
  let mut elements: usize = 0;
  for elem in iter {
    sw_cb(elem, vec)?;
    elements = elements.wrapping_add(1);
  }
  let value = value_cb(elements, vec, len_begin)?;
  write_len_end(len_write_begin, vec, value);
  Ok(())
}

#[inline]
fn writer_len_begin<E, T, const N: usize>(prefix: Option<u8>, vec: &mut T) -> Result<usize, E>
where
  E: From<crate::Error>,
  T: Lease<[u8]> + for<'any> TryExtend<&'any [u8]>,
{
  if let Some(elem) = prefix {
    vec.try_extend(&[elem])?;
  }
  let after_prefix = vec.lease().len();
  vec.try_extend(&[0; N])?;
  Ok(after_prefix)
}

#[inline]
fn write_len_end<T, const N: usize>(len_write_begin: usize, vec: &mut T, value: [u8; N])
where
  T: LeaseMut<[u8]>,
{
  let range = len_write_begin..len_write_begin.wrapping_add(N);
  let prefix_opt = vec.lease_mut().get_mut(range).and_then(|slice| slice.as_mut_array::<N>());
  // SAFETY: `start` and `value` are internally evaluated parameters that adhere to slice bounds.
  let prefix = unsafe { prefix_opt.unwrap_unchecked() };
  prefix.copy_from_slice(&value);
}

macro_rules! impl_trait {
  (($($name:ident)?), ($($name_iter:ident)?), $ty:ident, $bytes:literal, $max:literal) => {
    $(
      #[inline]
      pub(crate) fn $name<E, T>(
        cwbt: CounterWriterBytesTy,
        prefix: Option<u8>,
        vec: &mut T,
        cb: impl FnOnce(&mut T) -> Result<(), E>,
      ) -> Result<(), E>
      where
        E: From<crate::Error>,
        T: LeaseMut<[u8]> + for<'any> TryExtend<&'any [u8]>,
      {
        write(cwbt, prefix, vec, cb, |local_ew, len_begin| {
          let len = local_ew.lease().len().wrapping_sub(len_begin);
          if len > $max {
            return Err(crate::Error::CounterWriterOverflow.into());
          }
          let array = $ty::try_from(len).map_err(Into::into)?.to_be_bytes();
          let mut rslt = [0; $bytes];
          rslt.copy_from_slice(array.get(array.len().wrapping_sub($bytes)..).unwrap_or_default());
          Ok(rslt)
        })
      }
    )?

    $(
      #[inline]
      pub(crate) fn $name_iter<E, T, U>(
        cwit: CounterWriterIterTy,
        iter: impl IntoIterator<Item = U>,
        prefix: Option<u8>,
        vec: &mut T,
        cb: impl FnMut(U, &mut T) -> Result<(), E>,
      ) -> Result<(), E>
      where
        E: From<crate::Error>,
        T: LeaseMut<[u8]> + for<'any> TryExtend<&'any [u8]>,
      {
        write_iter(cwit, iter, prefix, vec, cb, |_counter, local_ew, len_begin| {
          let len = match cwit {
            CounterWriterIterTy::Bytes(_) => local_ew.lease().len().wrapping_sub(len_begin),
            #[cfg(feature = "postgres")]
            CounterWriterIterTy::Elements => _counter,
          };
          if len > $max {
            return Err(crate::Error::CounterWriterOverflow.into());
          }
          let array = $ty::try_from(len).map_err(Into::into)?.to_be_bytes();
          let mut rslt = [0; $bytes];
          rslt.copy_from_slice(array.get(array.len().wrapping_sub($bytes)..).unwrap_or_default());
          Ok(rslt)
        })
      }
    )?
  };
}

#[cfg(feature = "tls")]
impl_trait!((u8_write), (u8_write_iter), u8, 1, 255);
#[cfg(feature = "postgres")]
impl_trait!((), (i16_write_iter), i16, 2, 32_767);
#[cfg(feature = "tls")]
impl_trait!((u16_write), (u16_write_iter), u16, 2, 65_535);
#[cfg(feature = "tls")]
impl_trait!((u24_write), (u24_write_iter), u32, 3, 16_777_215);
#[cfg(feature = "postgres")]
impl_trait!((i32_write), (), i32, 4, 2_147_483_647);
