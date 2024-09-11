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
    ev: &mut D::EncodeValue<'buffer, 'tmp>,
    mut prefix_cb: impl FnMut(&mut A, &mut D::EncodeValue<'buffer, 'tmp>) -> usize,
    mut suffix_cb: impl FnMut(&mut A, &mut D::EncodeValue<'buffer, 'tmp>, bool, usize) -> usize,
  ) -> Result<usize, D::Error> {
    let mut n: usize = 0;
    for elem in *self {
      encode(aux, elem, ev, &mut n, &mut prefix_cb, &mut suffix_cb)?;
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

pub(crate) fn encode<'buffer, 'tmp, A, D, T>(
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
