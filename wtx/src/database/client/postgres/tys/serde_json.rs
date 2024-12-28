use crate::{
  database::{
    client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, PostgresError, Ty},
    Json, Typed,
  },
  misc::{Decode, Encode},
};
use serde::{Deserialize, Serialize};

impl<'de, E, T> Decode<'de, Postgres<E>> for Json<T>
where
  E: From<crate::Error>,
  T: Deserialize<'de>,
{
  #[inline]
  fn decode(input: &mut DecodeWrapper<'de>) -> Result<Self, E> {
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
  fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    ew.sw()._extend_from_byte(1).map_err(Into::into)?;
    serde_json::to_writer(ew.sw(), &self.0).map_err(Into::into)?;
    Ok(())
  }
}
impl<E, T> Typed<Postgres<E>> for Json<T>
where
  E: From<crate::Error>,
{
  const TY: Ty = Ty::Jsonb;
}
