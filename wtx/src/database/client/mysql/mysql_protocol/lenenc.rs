use crate::{
  database::client::mysql::{
    MysqlError,
    mysql_protocol::{MysqlProtocol, decode_wrapper_protocol::DecodeWrapperProtocol},
  },
  misc::Decode,
};

pub(crate) struct Lenenc(pub(crate) u64);

impl<DO, E> Decode<'_, MysqlProtocol<DO, E>> for Lenenc
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dw: &mut DecodeWrapperProtocol<'_, '_, DO>) -> Result<Self, E> {
    let [len, rest @ ..] = dw.bytes else {
      return Err(E::from(MysqlError::InvalidLenencBytes.into()));
    };
    let value = match *len {
      252 => {
        let [a, b, local_rest @ ..] = rest else {
          return Err(E::from(MysqlError::InvalidLenencBytes.into()));
        };
        *dw.bytes = local_rest;
        u16::from_le_bytes([*a, *b]).into()
      }
      253 => {
        let [a, b, c, local_rest @ ..] = rest else {
          return Err(E::from(MysqlError::InvalidLenencBytes.into()));
        };
        *dw.bytes = local_rest;
        u32::from_le_bytes([*a, *b, *c, 0]).into()
      }
      254 => {
        let [a, b, c, d, e, f, g, h, local_rest @ ..] = rest else {
          return Err(E::from(MysqlError::InvalidLenencBytes.into()));
        };
        *dw.bytes = local_rest;
        u64::from_le_bytes([*a, *b, *c, *d, *e, *f, *g, *h])
      }
      n => {
        *dw.bytes = rest;
        u64::from(n)
      }
    };
    Ok(Self(value))
  }
}
