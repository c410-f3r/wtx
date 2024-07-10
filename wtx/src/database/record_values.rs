#![expect(non_snake_case, reason = "meta variable expressions")]

use crate::{
  database::{Database, Encode},
  misc::{into_rslt, FilledBufferWriter, IterWrapper},
};

/// Values that can passed to a record as parameters. For example, in a query.
pub trait RecordValues<D>
where
  D: Database,
{
  /// Converts the inner values into a byte representation.
  fn encode_values<'ev, A>(
    &mut self,
    aux: &mut A,
    fbw: &mut FilledBufferWriter<'_>,
    values: impl Iterator<Item = &'ev D::EncodeValue<'ev>> + 'ev,
    prefix_cb: impl FnMut(&mut A, &mut FilledBufferWriter<'_>) -> usize,
    suffix_cb: impl FnMut(&mut A, &mut FilledBufferWriter<'_>, bool) -> usize,
  ) -> Result<usize, D::Error>
  where
    <D as Database>::EncodeValue<'ev>: 'ev;

  /// The number of values
  fn len(&self) -> usize;
}

impl<D> RecordValues<D> for ()
where
  D: Database,
{
  #[inline]
  fn encode_values<'ev, A>(
    &mut self,
    _: &mut A,
    _: &mut FilledBufferWriter<'_>,
    _: impl Iterator<Item = &'ev D::EncodeValue<'ev>> + 'ev,
    _: impl FnMut(&mut A, &mut FilledBufferWriter<'_>) -> usize,
    _: impl FnMut(&mut A, &mut FilledBufferWriter<'_>, bool) -> usize,
  ) -> Result<usize, D::Error>
  where
    <D as Database>::EncodeValue<'ev>: 'ev,
  {
    Ok(0)
  }

  #[inline]
  fn len(&self) -> usize {
    0
  }
}

impl<D, T> RecordValues<D> for &mut T
where
  D: Database,
  T: RecordValues<D>,
{
  #[inline]
  fn encode_values<'ev, A>(
    &mut self,
    aux: &mut A,
    fbw: &mut FilledBufferWriter<'_>,
    values: impl Iterator<Item = &'ev D::EncodeValue<'ev>> + 'ev,
    prefix_cb: impl FnMut(&mut A, &mut FilledBufferWriter<'_>) -> usize,
    suffix_cb: impl FnMut(&mut A, &mut FilledBufferWriter<'_>, bool) -> usize,
  ) -> Result<usize, D::Error>
  where
    <D as Database>::EncodeValue<'ev>: 'ev,
  {
    (**self).encode_values(aux, fbw, values, prefix_cb, suffix_cb)
  }

  #[inline]
  fn len(&self) -> usize {
    (**self).len()
  }
}

impl<D, T> RecordValues<D> for &[T]
where
  D: Database,
  T: Encode<D>,
{
  #[inline]
  fn encode_values<'ev, A>(
    &mut self,
    aux: &mut A,
    fbw: &mut FilledBufferWriter<'_>,
    values: impl Iterator<Item = &'ev D::EncodeValue<'ev>> + 'ev,
    mut prefix_cb: impl FnMut(&mut A, &mut FilledBufferWriter<'_>) -> usize,
    mut suffix_cb: impl FnMut(&mut A, &mut FilledBufferWriter<'_>, bool) -> usize,
  ) -> Result<usize, D::Error>
  where
    <D as Database>::EncodeValue<'ev>: 'ev,
  {
    let mut n: usize = 0;
    for (elem, value) in self.iter().zip(values) {
      encode(aux, elem, fbw, &mut n, value, &mut prefix_cb, &mut suffix_cb)?;
    }
    Ok(n)
  }

  #[inline]
  fn len(&self) -> usize {
    (*self).len()
  }
}

impl<D, I, T> RecordValues<D> for IterWrapper<I>
where
  D: Database,
  I: Iterator<Item = T>,
  T: Encode<D>,
{
  #[inline]
  fn encode_values<'ev, A>(
    &mut self,
    aux: &mut A,
    fbw: &mut FilledBufferWriter<'_>,
    values: impl Iterator<Item = &'ev D::EncodeValue<'ev>> + 'ev,
    mut prefix_cb: impl FnMut(&mut A, &mut FilledBufferWriter<'_>) -> usize,
    mut suffix_cb: impl FnMut(&mut A, &mut FilledBufferWriter<'_>, bool) -> usize,
  ) -> Result<usize, D::Error>
  where
    <D as Database>::EncodeValue<'ev>: 'ev,
  {
    let mut n: usize = 0;
    for (elem, value) in self.0.by_ref().zip(values) {
      encode(aux, &elem, fbw, &mut n, value, &mut prefix_cb, &mut suffix_cb)?;
    }
    Ok(n)
  }

  #[inline]
  fn len(&self) -> usize {
    let (l, u) = self.0.size_hint();
    u.unwrap_or(l)
  }
}

macro_rules! tuple_impls {
  ($( ($($T:ident)+) )+) => {
    $(
      impl<DB, $($T),+> RecordValues<DB> for ($( $T, )+)
      where
        DB: Database,
        $($T: Encode<DB>,)+
      {
        #[inline]
        fn encode_values<'ev, AUX>(
          &mut self,
          aux: &mut AUX,
          fbw: &mut FilledBufferWriter<'_>,
          mut values: impl Iterator<Item = &'ev DB::EncodeValue<'ev>> + 'ev,
          mut prefix_cb: impl FnMut(&mut AUX, &mut FilledBufferWriter<'_>) -> usize,
          mut suffix_cb: impl FnMut(&mut AUX, &mut FilledBufferWriter<'_>, bool) -> usize,
        ) -> Result<usize, DB::Error>
        where
          <DB as Database>::EncodeValue<'ev>: 'ev
        {
          let mut n: usize = 0;
          let ($($T,)+) = self;
          $(
            encode(
              aux,
              $T,
              fbw,
              &mut n,
              into_rslt(values.next())?,
              &mut prefix_cb,
              &mut suffix_cb
            )?;
          )+
          Ok(n)
        }

        #[inline]
        fn len(&self) -> usize {
          let mut len: usize = 0;
          $({ const $T: usize = 1; len = len.wrapping_add($T); })+
          len
        }
      }
    )+
  }
}

tuple_impls! {
  (A)
  (A B)
  (A B C)
  (A B C D)
  (A B C D E)
  (A B C D E F)
  (A B C D E F G)
  (A B C D E F G H)
  (A B C D E F G H I)
  (A B C D E F G H I J)
  (A B C D E F G H I J K)
  (A B C D E F G H I J K L)
  (A B C D E F G H I J K L M)
  (A B C D E F G H I J K L M N)
  (A B C D E F G H I J K L M N O)
  (A B C D E F G H I J K L M N O P)
}

fn encode<A, D, T>(
  aux: &mut A,
  elem: &T,
  fbw: &mut FilledBufferWriter<'_>,
  n: &mut usize,
  value: &D::EncodeValue<'_>,
  prefix_cb: &mut impl FnMut(&mut A, &mut FilledBufferWriter<'_>) -> usize,
  suffix_cb: &mut impl FnMut(&mut A, &mut FilledBufferWriter<'_>, bool) -> usize,
) -> Result<(), D::Error>
where
  D: Database,
  T: Encode<D>,
{
  *n = n.wrapping_add(prefix_cb(aux, fbw));
  let before = fbw._len();
  elem.encode(fbw, value)?;
  *n = n.wrapping_add(fbw._len().wrapping_sub(before));
  *n = n.wrapping_add(suffix_cb(aux, fbw, elem.is_null()));
  Ok(())
}
