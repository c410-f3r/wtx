use crate::{
  database::{
    Json, Typed,
    client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, PostgresError, Ty},
  },
  de::{Decode, Encode},
  misc::serde_json_deserialize_from_slice,
};
use serde::{Deserialize, Serialize};

impl<'de, E, T> Decode<'de, Postgres<E>> for Json<T>
where
  E: From<crate::Error>,
  T: Deserialize<'de>,
{
  #[inline]
  fn decode(input: &mut DecodeWrapper<'de, '_>) -> Result<Self, E> {
    let [1, rest @ ..] = input.bytes() else {
      return Err(E::from(PostgresError::InvalidJsonFormat.into()));
    };
    let elem = serde_json_deserialize_from_slice(rest)?;
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
