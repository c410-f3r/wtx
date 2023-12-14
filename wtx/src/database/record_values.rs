use crate::{database::Encode, misc::FilledBufferWriter};

/// Values that can passed to a record as parameters. For example, in a query.
pub trait RecordValues<C, E>
where
  E: From<crate::Error>,
{
  /// Converts the inner values into a byte representation.
  fn encode_values<A>(
    self,
    aux: &mut A,
    fbw: &mut FilledBufferWriter<'_>,
    prefix_cb: impl FnMut(&mut A, &mut FilledBufferWriter<'_>) -> usize,
    suffix_cb: impl FnMut(&mut A, &mut FilledBufferWriter<'_>, bool) -> usize,
  ) -> Result<usize, E>;

  /// The number of values
  fn len(&self) -> usize;
}

impl<C> RecordValues<C, crate::Error> for () {
  #[inline]
  fn encode_values<A>(
    self,
    _: &mut A,
    _: &mut FilledBufferWriter<'_>,
    _: impl FnMut(&mut A, &mut FilledBufferWriter<'_>) -> usize,
    _: impl FnMut(&mut A, &mut FilledBufferWriter<'_>, bool) -> usize,
  ) -> Result<usize, crate::Error> {
    Ok(0)
  }

  #[inline]
  fn len(&self) -> usize {
    0
  }
}

impl<C, E, T> RecordValues<C, E> for &T
where
  E: From<crate::Error>,
  T: Encode<C, E>,
{
  #[inline]
  fn encode_values<A>(
    self,
    aux: &mut A,
    fbw: &mut FilledBufferWriter<'_>,
    mut prefix_cb: impl FnMut(&mut A, &mut FilledBufferWriter<'_>) -> usize,
    mut suffix_cb: impl FnMut(&mut A, &mut FilledBufferWriter<'_>, bool) -> usize,
  ) -> Result<usize, E> {
    let mut n: usize = 0;
    encode(aux, self, fbw, &mut n, &mut prefix_cb, &mut suffix_cb)?;
    Ok(n)
  }

  #[inline]
  fn len(&self) -> usize {
    1
  }
}

impl<C, E, T> RecordValues<C, E> for &[T]
where
  E: From<crate::Error>,
  T: Encode<C, E>,
{
  #[inline]
  fn encode_values<A>(
    self,
    aux: &mut A,
    fbw: &mut FilledBufferWriter<'_>,
    mut prefix_cb: impl FnMut(&mut A, &mut FilledBufferWriter<'_>) -> usize,
    mut suffix_cb: impl FnMut(&mut A, &mut FilledBufferWriter<'_>, bool) -> usize,
  ) -> Result<usize, E> {
    let mut n: usize = 0;
    for elem in self {
      encode(aux, elem, fbw, &mut n, &mut prefix_cb, &mut suffix_cb)?;
    }
    Ok(n)
  }

  #[inline]
  fn len(&self) -> usize {
    (*self).len()
  }
}

impl<C, E, I, T> RecordValues<C, E> for &mut I
where
  E: From<crate::Error>,
  I: ExactSizeIterator<Item = T>,
  T: Encode<C, E>,
{
  #[inline]
  fn encode_values<A>(
    self,
    aux: &mut A,
    fbw: &mut FilledBufferWriter<'_>,
    mut prefix_cb: impl FnMut(&mut A, &mut FilledBufferWriter<'_>) -> usize,
    mut suffix_cb: impl FnMut(&mut A, &mut FilledBufferWriter<'_>, bool) -> usize,
  ) -> Result<usize, E> {
    let mut n: usize = 0;
    for elem in self {
      encode(aux, &elem, fbw, &mut n, &mut prefix_cb, &mut suffix_cb)?;
    }
    Ok(n)
  }

  #[inline]
  fn len(&self) -> usize {
    (**self).len()
  }
}

macro_rules! tuple_impls {
  ($(
    $tuple_len:tt {
      $(($idx:tt) -> $T:ident)+
    }
  )+) => {
    $(
      impl<CTX, ERR, $($T),+> RecordValues<CTX, ERR> for ($( $T, )+)
      where
        ERR: From<crate::Error>,
        $($T: Encode<CTX, ERR>,)+
      {
        #[inline]
        fn encode_values<AUX>(
          self,
          aux: &mut AUX,
          fbw: &mut FilledBufferWriter<'_>,
          mut prefix_cb: impl FnMut(&mut AUX, &mut FilledBufferWriter<'_>) -> usize,
          mut suffix_cb: impl FnMut(&mut AUX, &mut FilledBufferWriter<'_>, bool) -> usize,
        ) -> Result<usize, ERR> {
          let mut n: usize = 0;
          $( encode(aux, &self.$idx, fbw, &mut n, &mut prefix_cb, &mut suffix_cb)?; )+
          Ok(n)
        }

        #[inline]
        fn len(&self) -> usize {
          $tuple_len
        }
      }
    )+
  }
}

tuple_impls! {
  1 {
    (0) -> A
  }
  2 {
    (0) -> A
    (1) -> B
  }
  3 {
    (0) -> A
    (1) -> B
    (2) -> C
  }
  4 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
  }
  5 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
  }
  6 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
  }
  7 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
  }
  8 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
    (7) -> H
  }
  9 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
    (7) -> H
    (8) -> I
  }
  10 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
    (7) -> H
    (8) -> I
    (9) -> J
  }
  11 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
    (7) -> H
    (8) -> I
    (9) -> J
    (10) -> K
  }
  12 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
    (7) -> H
    (8) -> I
    (9) -> J
    (10) -> K
    (11) -> L
  }
  13 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
    (7) -> H
    (8) -> I
    (9) -> J
    (10) -> K
    (11) -> L
    (12) -> M
  }
  14 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
    (7) -> H
    (8) -> I
    (9) -> J
    (10) -> K
    (11) -> L
    (12) -> M
    (13) -> N
  }
  15 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
    (7) -> H
    (8) -> I
    (9) -> J
    (10) -> K
    (11) -> L
    (12) -> M
    (13) -> N
    (14) -> O
  }
  16 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
    (7) -> H
    (8) -> I
    (9) -> J
    (10) -> K
    (11) -> L
    (12) -> M
    (13) -> N
    (14) -> O
    (15) -> P
  }
}

fn encode<A, C, E, T>(
  aux: &mut A,
  elem: &T,
  fbw: &mut FilledBufferWriter<'_>,
  n: &mut usize,
  prefix_cb: &mut impl FnMut(&mut A, &mut FilledBufferWriter<'_>) -> usize,
  suffix_cb: &mut impl FnMut(&mut A, &mut FilledBufferWriter<'_>, bool) -> usize,
) -> Result<(), E>
where
  E: From<crate::Error>,
  T: Encode<C, E>,
{
  *n = n.wrapping_add(prefix_cb(aux, fbw));
  let before = fbw._len();
  elem.encode(fbw)?;
  *n = n.wrapping_add(fbw._len().wrapping_sub(before));
  *n = n.wrapping_add(suffix_cb(aux, fbw, elem.is_null()));
  Ok(())
}
