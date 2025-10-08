use crate::{
  database::{
    Json, Typed,
    client::postgres::{Postgres, PostgresDecodeWrapper, PostgresEncodeWrapper, PostgresError, Ty},
  },
  de::{Decode, Encode},
};
use serde::{Deserialize, Serialize};

impl<'de, E, T> Decode<'de, Postgres<E>> for Json<T>
where
  E: From<crate::Error>,
  T: Deserialize<'de>,
{
  #[inline]
  fn decode(_: &mut (), input: &mut PostgresDecodeWrapper<'de, '_>) -> Result<Self, E> {
    let [1, rest @ ..] = input.bytes() else {
      return Err(E::from(PostgresError::InvalidJsonFormat.into()));
    };
    let elem = serde_json::from_slice(rest).map(Json).map_err(Into::into)?;
    Ok(elem)
  }
}
impl<E, T> Encode<Postgres<E>> for Json<T>
where
  E: From<crate::Error>,
  T: Serialize,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut PostgresEncodeWrapper<'_, '_>) -> Result<(), E> {
    ew.buffer().extend_from_byte(1)?;
    serde_json::to_writer(ew.buffer(), &self.0).map_err(Into::into)?;
    Ok(())
  }
}
impl<E, T> Typed<Postgres<E>> for Json<T>
where
  E: From<crate::Error>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    <Self as Typed<Postgres<E>>>::static_ty()
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    Some(Ty::Jsonb)
  }
}
