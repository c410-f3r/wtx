use crate::{
  collection::Vector,
  misc::{LeaseMut, SuffixWriter},
};

#[derive(Clone, Copy, Debug)]
pub(crate) enum CounterWriterIterTy {
  Bytes(CounterWriterBytesTy),
  Elements,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum CounterWriterBytesTy {
  IncludesLen,
  IgnoresLen,
}

#[inline]
fn write<E, SW, V, const N: usize>(
  cwbt: CounterWriterBytesTy,
  prefix: Option<u8>,
  sw: &mut SW,
  sw_cb: impl FnOnce(&mut SW) -> Result<(), E>,
  value_cb: impl FnOnce(&mut SW, usize) -> Result<[u8; N], E>,
) -> Result<(), E>
where
  E: From<crate::Error>,
  SW: LeaseMut<SuffixWriter<V>>,
  V: LeaseMut<Vector<u8>>,
{
  let len_write_begin = writer_len_begin::<_, _, _, N>(sw, prefix)?;
  let len_begin = match cwbt {
    CounterWriterBytesTy::IgnoresLen => sw.lease().len(),
    CounterWriterBytesTy::IncludesLen => len_write_begin,
  };
  sw_cb(sw)?;
  let value = value_cb(sw, len_begin)?;
  Ok(write_len_end(len_write_begin, sw, value))
}

#[inline]
fn write_iter<E, SW, T, V, const N: usize>(
  cwit: CounterWriterIterTy,
  iter: impl IntoIterator<Item = T>,
  prefix: Option<u8>,
  sw: &mut SW,
  mut sw_cb: impl FnMut(T, &mut SW) -> Result<(), E>,
  value_cb: impl FnOnce(usize, &mut SW, usize) -> Result<[u8; N], E>,
) -> Result<(), E>
where
  E: From<crate::Error>,
  SW: LeaseMut<SuffixWriter<V>>,
  V: LeaseMut<Vector<u8>>,
{
  let len_write_begin = writer_len_begin::<_, _, _, N>(sw, prefix)?;
  let len_begin = match cwit {
    CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen) => sw.lease().len(),
    CounterWriterIterTy::Bytes(CounterWriterBytesTy::IncludesLen) => len_write_begin,
    CounterWriterIterTy::Elements => 0,
  };
  let mut elements: usize = 0;
  for elem in iter.into_iter() {
    sw_cb(elem, sw)?;
    elements = elements.wrapping_add(1);
  }
  let value = value_cb(elements, sw, len_begin)?;
  Ok(write_len_end(len_write_begin, sw, value))
}

#[inline]
fn writer_len_begin<E, SW, V, const N: usize>(sw: &mut SW, prefix: Option<u8>) -> Result<usize, E>
where
  E: From<crate::Error>,
  SW: LeaseMut<SuffixWriter<V>>,
  V: LeaseMut<Vector<u8>>,
{
  if let Some(elem) = prefix {
    sw.lease_mut().extend_from_byte(elem)?;
  }
  let after_prefix = sw.lease().len();
  sw.lease_mut().extend_from_slice(&[0; N])?;
  Ok(after_prefix)
}

#[inline]
fn write_len_end<SW, V, const N: usize>(len_write_begin: usize, sw: &mut SW, value: [u8; N])
where
  SW: LeaseMut<SuffixWriter<V>>,
  V: LeaseMut<Vector<u8>>,
{
  let range = len_write_begin..len_write_begin.wrapping_add(N);
  let prefix_opt =
    sw.lease_mut().curr_bytes_mut().get_mut(range).and_then(|slice| slice.as_mut_array::<N>());
  // SAFETY: `start` and `value` are internally evaluated parameters that adhere to slice bounds.
  let prefix = unsafe { prefix_opt.unwrap_unchecked() };
  prefix.copy_from_slice(&value);
}

macro_rules! impl_trait {
  ($name:ident, $name_iter:ident, $ty:ident, $bytes:literal, $max:literal) => {
    #[inline]
    pub(crate) fn $name<E, SW, V>(
      cwbt: CounterWriterBytesTy,
      prefix: Option<u8>,
      sw: &mut SW,
      cb: impl FnOnce(&mut SW) -> Result<(), E>,
    ) -> Result<(), E>
    where
      E: From<crate::Error>,
      SW: LeaseMut<SuffixWriter<V>>,
      V: LeaseMut<Vector<u8>>,
    {
      write(cwbt, prefix, sw, cb, |local_sw, len_begin| {
        let len = local_sw.lease().len().wrapping_sub(len_begin);
        if len > $max {
          return Err(crate::Error::CounterWriterOverflow.into());
        }
        let array = $ty::try_from(len).map_err(Into::into)?.to_be_bytes();
        let mut rslt = [0; $bytes];
        rslt.copy_from_slice(&array[array.len() - $bytes..]);
        Ok(rslt)
      })
    }

    #[inline]
    pub(crate) fn $name_iter<E, SW, T, V>(
      cwit: CounterWriterIterTy,
      iter: impl IntoIterator<Item = T>,
      prefix: Option<u8>,
      sw: &mut SW,
      cb: impl FnMut(T, &mut SW) -> Result<(), E>,
    ) -> Result<(), E>
    where
      E: From<crate::Error>,
      SW: LeaseMut<SuffixWriter<V>>,
      V: LeaseMut<Vector<u8>>,
    {
      write_iter(cwit, iter, prefix, sw, cb, |counter, local_sw, len_begin| {
        let len = match cwit {
          CounterWriterIterTy::Bytes(_) => local_sw.lease().len().wrapping_sub(len_begin),
          CounterWriterIterTy::Elements => counter,
        };
        if len > $max {
          return Err(crate::Error::CounterWriterOverflow.into());
        }
        let array = $ty::try_from(len).map_err(Into::into)?.to_be_bytes();
        let mut rslt = [0; $bytes];
        rslt.copy_from_slice(&array[array.len() - $bytes..]);
        Ok(rslt)
      })
    }
  };
}

#[cfg(feature = "tls")]
impl_trait!(u8_write, u8_write_iter, u8, 1, 255);
#[cfg(feature = "postgres")]
impl_trait!(i16_write, i16_write_iter, i16, 2, 32_767);
#[cfg(feature = "tls")]
impl_trait!(u16_write, u16_write_iter, u16, 2, 65_535);
#[cfg(feature = "tls")]
impl_trait!(u24_write, u24_write_iter, u32, 3, 16_777_215);
#[cfg(feature = "postgres")]
impl_trait!(i32_write, i32_write_iter, i32, 4, 2_147_483_647);
