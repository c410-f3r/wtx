use crate::{
  database::{Database, Typed},
  misc::{Encode, Lease, Wrapper},
};

/// Values that can passed to a record as parameters. For example, in a query.
pub trait RecordValues<D>
where
  D: Database,
{
  /// Converts the inner values into a byte representation.
  fn encode_values<'buffer, 'tmp, A>(
    &self,
    aux: &mut A,
    ew: &mut D::EncodeWrapper<'buffer, 'tmp>,
    prefix_cb: impl FnMut(&mut A, &mut D::EncodeWrapper<'buffer, 'tmp>) -> usize,
    suffix_cb: impl FnMut(&mut A, &mut D::EncodeWrapper<'buffer, 'tmp>, bool, usize) -> usize,
  ) -> Result<usize, D::Error>
  where
    'buffer: 'tmp;

  /// The number of values
  fn len(&self) -> usize;

  /// Walks through all elements showing their types as well as if they are null.
  fn walk(
    &self,
    cb: impl FnMut(bool, Option<D::Ty>) -> Result<(), D::Error>,
  ) -> Result<(), D::Error>;
}

impl<D, T> RecordValues<D> for &mut T
where
  D: Database,
  T: RecordValues<D>,
{
  #[inline]
  fn encode_values<'buffer, 'tmp, A>(
    &self,
    aux: &mut A,
    ew: &mut D::EncodeWrapper<'buffer, 'tmp>,
    prefix_cb: impl FnMut(&mut A, &mut D::EncodeWrapper<'buffer, 'tmp>) -> usize,
    suffix_cb: impl FnMut(&mut A, &mut D::EncodeWrapper<'buffer, 'tmp>, bool, usize) -> usize,
  ) -> Result<usize, D::Error> {
    (**self).encode_values(aux, ew, prefix_cb, suffix_cb)
  }

  #[inline]
  fn len(&self) -> usize {
    (**self).len()
  }

  #[inline]
  fn walk(
    &self,
    cb: impl FnMut(bool, Option<D::Ty>) -> Result<(), D::Error>,
  ) -> Result<(), D::Error> {
    (**self).walk(cb)
  }
}

impl<D, T> RecordValues<D> for &[T]
where
  D: Database<Aux = ()>,
  T: Encode<D> + Typed<D>,
{
  #[inline]
  fn encode_values<'buffer, 'tmp, A>(
    &self,
    aux: &mut A,
    ew: &mut D::EncodeWrapper<'buffer, 'tmp>,
    mut prefix_cb: impl FnMut(&mut A, &mut D::EncodeWrapper<'buffer, 'tmp>) -> usize,
    mut suffix_cb: impl FnMut(&mut A, &mut D::EncodeWrapper<'buffer, 'tmp>, bool, usize) -> usize,
  ) -> Result<usize, D::Error> {
    let mut n: usize = 0;
    for elem in *self {
      encode(aux, elem, ew, &mut n, &mut prefix_cb, &mut suffix_cb)?;
    }
    Ok(n)
  }

  #[inline]
  fn len(&self) -> usize {
    (*self).len()
  }

  #[inline]
  fn walk(
    &self,
    mut cb: impl FnMut(bool, Option<D::Ty>) -> Result<(), D::Error>,
  ) -> Result<(), D::Error> {
    for elem in *self {
      cb(elem.is_null(), T::TY)?;
    }
    Ok(())
  }
}

impl<D, I, T> RecordValues<D> for Wrapper<I>
where
  D: Database<Aux = ()>,
  I: Clone + Iterator<Item = T>,
  T: Encode<D> + Typed<D>,
{
  #[inline]
  fn encode_values<'buffer, 'tmp, A>(
    &self,
    aux: &mut A,
    ew: &mut D::EncodeWrapper<'buffer, 'tmp>,
    mut prefix_cb: impl FnMut(&mut A, &mut D::EncodeWrapper<'buffer, 'tmp>) -> usize,
    mut suffix_cb: impl FnMut(&mut A, &mut D::EncodeWrapper<'buffer, 'tmp>, bool, usize) -> usize,
  ) -> Result<usize, D::Error> {
    let mut n: usize = 0;
    for elem in self.0.clone() {
      encode(aux, &elem, ew, &mut n, &mut prefix_cb, &mut suffix_cb)?;
    }
    Ok(n)
  }

  #[inline]
  fn len(&self) -> usize {
    let (l, u) = self.0.size_hint();
    u.unwrap_or(l)
  }

  #[inline]
  fn walk(
    &self,
    mut cb: impl FnMut(bool, Option<D::Ty>) -> Result<(), D::Error>,
  ) -> Result<(), D::Error> {
    for elem in self.0.clone() {
      cb(elem.is_null(), T::TY)?;
    }
    Ok(())
  }
}

pub(crate) fn encode<'buffer, 'tmp, A, D, T>(
  aux: &mut A,
  elem: &T,
  ew: &mut D::EncodeWrapper<'buffer, 'tmp>,
  n: &mut usize,
  prefix_cb: &mut impl FnMut(&mut A, &mut D::EncodeWrapper<'buffer, 'tmp>) -> usize,
  suffix_cb: &mut impl FnMut(&mut A, &mut D::EncodeWrapper<'buffer, 'tmp>, bool, usize) -> usize,
) -> Result<(), D::Error>
where
  D: Database<Aux = ()>,
  T: Encode<D>,
{
  *n = n.wrapping_add(prefix_cb(aux, ew));
  let elem_before = ew.lease().len();
  elem.encode(&mut (), ew)?;
  let elem_len = ew.lease().len().wrapping_sub(elem_before);
  *n = n.wrapping_add(elem_len);
  *n = n.wrapping_add(suffix_cb(aux, ew, elem.is_null(), elem_len));
  Ok(())
}
