use crate::{
  codec::{Decode, Encode},
  database::{Database, Record, RecordValues, Typed},
};

/// User-defined data that is persisted alongside a session.
pub trait SessionCustomState<D>: Sized
where
  D: Database,
{
  /// Identifiers of the columns that compose the custom state, in the same order as the values
  /// returned by [`Self::values`].
  const IDENTIFIERS: &'static [&'static str];
  /// Usually a database schema prefix.
  const NAMESPACE: &str;

  /// New instance from external parameters.
  fn new<'exec, R>(record: &R) -> Result<Self, D::Error>
  where
    R: Record<'exec, Database = D>;

  /// Values for [`Self::IDENTIFIERS`], in the same order.
  fn values(&self) -> impl RecordValues<D>;
}

impl<D> SessionCustomState<D> for ()
where
  D: Database,
{
  const IDENTIFIERS: &'static [&'static str] = &[];
  const NAMESPACE: &str = "_wtx";

  #[inline]
  fn new<'exec, R>(_: &R) -> Result<Self, D::Error>
  where
    R: Record<'exec, Database = D>,
  {
    Ok(())
  }

  #[inline]
  fn values(&self) -> impl RecordValues<D> {}
}

macro_rules! impl_session_custom_state {
  ($($ty:ty),*) => {
    $(
      impl<D> SessionCustomState<D> for $ty
      where
        D: Database,
        $ty: Encode<D> + Typed<D>,
        for<'exec> $ty: Decode<'exec, D>,
      {
        const IDENTIFIERS: &'static [&'static str] = &["custom_state"];
        const NAMESPACE: &str = "_wtx";

        #[inline]
        fn new<'exec, R>(record: &R) -> Result<Self, D::Error>
        where
          R: Record<'exec, Database = D>,
        {
          record.decode(4)
        }

        #[inline]
        fn values(&self) -> impl RecordValues<D> {
          (self,)
        }
      }
    )*
  };
}

impl_session_custom_state!(i32, i64, u32, u64);
