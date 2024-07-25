use crate::{
  database::{Database, Encode},
  misc::{IterWrapper, Lease},
};

/// Values that can passed to a record as parameters. For example, in a query.
pub trait RecordValues<D>
where
  D: Database,
{
  /// Converts the inner values into a byte representation.
  fn encode_values<'buffer, 'tmp, A>(
    &mut self,
    aux: &mut A,
    ev: &mut D::EncodeValue<'buffer, 'tmp>,
    prefix_cb: impl FnMut(&mut A, &mut D::EncodeValue<'buffer, 'tmp>) -> usize,
    suffix_cb: impl FnMut(&mut A, &mut D::EncodeValue<'buffer, 'tmp>, bool, usize) -> usize,
  ) -> Result<usize, D::Error>
  where
    'buffer: 'tmp;

  /// The number of values
  fn len(&self) -> usize;
}

impl<D> RecordValues<D> for ()
where
  D: Database,
{
  #[inline]
  fn encode_values<'buffer, 'tmp, A>(
    &mut self,
    _: &mut A,
    _: &mut D::EncodeValue<'buffer, 'tmp>,
    _: impl FnMut(&mut A, &mut D::EncodeValue<'buffer, 'tmp>) -> usize,
    _: impl FnMut(&mut A, &mut D::EncodeValue<'buffer, 'tmp>, bool, usize) -> usize,
  ) -> Result<usize, D::Error> {
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
  fn encode_values<'buffer, 'tmp, A>(
    &mut self,
    aux: &mut A,
    ev: &mut D::EncodeValue<'buffer, 'tmp>,
    prefix_cb: impl FnMut(&mut A, &mut D::EncodeValue<'buffer, 'tmp>) -> usize,
    suffix_cb: impl FnMut(&mut A, &mut D::EncodeValue<'buffer, 'tmp>, bool, usize) -> usize,
  ) -> Result<usize, D::Error> {
    (**self).encode_values(aux, ev, prefix_cb, suffix_cb)
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
  fn encode_values<'buffer, 'tmp, A>(
    &mut self,
    aux: &mut A,
    mut ev: &mut D::EncodeValue<'buffer, 'tmp>,
    mut prefix_cb: impl FnMut(&mut A, &mut D::EncodeValue<'buffer, 'tmp>) -> usize,
    mut suffix_cb: impl FnMut(&mut A, &mut D::EncodeValue<'buffer, 'tmp>, bool, usize) -> usize,
  ) -> Result<usize, D::Error> {
    let mut n: usize = 0;
    for elem in self.iter() {
      encode(aux, elem, &mut ev, &mut n, &mut prefix_cb, &mut suffix_cb)?;
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
  fn encode_values<'buffer, 'tmp, A>(
    &mut self,
    aux: &mut A,
    ev: &mut D::EncodeValue<'buffer, 'tmp>,
    mut prefix_cb: impl FnMut(&mut A, &mut D::EncodeValue<'buffer, 'tmp>) -> usize,
    mut suffix_cb: impl FnMut(&mut A, &mut D::EncodeValue<'buffer, 'tmp>, bool, usize) -> usize,
  ) -> Result<usize, D::Error> {
    let mut n: usize = 0;
    for elem in self.0.by_ref() {
      encode(aux, &elem, ev, &mut n, &mut prefix_cb, &mut suffix_cb)?;
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
      #[expect(non_snake_case, reason = "meta variable expressions")]
      impl<DB, $($T),+> RecordValues<DB> for ($( $T, )+)
      where
        DB: Database,
        $($T: Encode<DB>,)+
      {
        #[inline]
        fn encode_values<'buffer, 'tmp, AUX>(
          &mut self,
          aux: &mut AUX,
          ev: &mut DB::EncodeValue<'buffer, 'tmp>,
          mut prefix_cb: impl FnMut(&mut AUX, &mut DB::EncodeValue<'buffer, 'tmp>) -> usize,
          mut suffix_cb: impl FnMut(&mut AUX, &mut DB::EncodeValue<'buffer, 'tmp>, bool, usize) -> usize,
        ) -> Result<usize, DB::Error> {
          let mut n: usize = 0;
          let ($($T,)+) = self;
          $(
            encode(
              aux,
              $T,
              ev,
              &mut n,
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

fn encode<'buffer, 'tmp, A, D, T>(
  aux: &mut A,
  elem: &T,
  ev: &mut D::EncodeValue<'buffer, 'tmp>,
  n: &mut usize,
  prefix_cb: &mut impl FnMut(&mut A, &mut D::EncodeValue<'buffer, 'tmp>) -> usize,
  suffix_cb: &mut impl FnMut(&mut A, &mut D::EncodeValue<'buffer, 'tmp>, bool, usize) -> usize,
) -> Result<(), D::Error>
where
  D: Database,
  T: Encode<D>,
{
  *n = n.wrapping_add(prefix_cb(aux, ev));
  let elem_before = ev.lease()._len();
  elem.encode(ev)?;
  let elem_len = ev.lease()._len().wrapping_sub(elem_before);
  *n = n.wrapping_add(elem_len);
  *n = n.wrapping_add(suffix_cb(aux, ev, elem.is_null(), elem_len));
  Ok(())
}
