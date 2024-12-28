use crate::{
  database::{
    client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, Ty},
    Typed,
  },
  misc::{Decode, Encode},
};
use uuid::Uuid;

impl<'de, E> Decode<'de, Postgres<E>> for Uuid
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> Result<Self, E> {
    let elem = Uuid::from_slice(dw.bytes()).map_err(Into::into)?;
    Ok(elem)
  }
}

impl<E> Encode<Postgres<E>> for Uuid
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    ew.sw().extend_from_slice(self.as_bytes()).map_err(Into::into)?;
    Ok(())
  }
}

impl<E> Typed<Postgres<E>> for Uuid
where
  E: From<crate::Error>,
{
  const TY: Ty = Ty::Uuid;
}

test!(uuid, Uuid, Uuid::max());
