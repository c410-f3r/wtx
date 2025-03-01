use crate::{
  database::client::mysql::{
    MysqlError,
    mysql_protocol::{
      MysqlProtocol, decode_wrapper_protocol::DecodeWrapperProtocol, lenenc::Lenenc,
    },
    status::Status,
  },
  misc::Decode,
};

#[derive(Debug)]
pub(crate) struct OkRes {
  pub(crate) affected_rows: u64,
  pub(crate) status: Status,
}

impl<DO, E> Decode<'_, MysqlProtocol<DO, E>> for OkRes
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dw: &mut DecodeWrapperProtocol<'_, '_, DO>) -> Result<Self, E> {
    let [header, rest @ ..] = dw.bytes else {
      return Err(E::from(MysqlError::InvalidOkBytes.into()));
    };
    if *header != 0 && *header != 254 {
      return Err(E::from(MysqlError::InvalidOkBytes.into()));
    }
    *dw.bytes = rest;
    let affected_rows = Lenenc::decode(&mut (), dw)?.0;
    let _last_insert_id = Lenenc::decode(&mut (), dw)?.0;
    let [a, b, c, d, rest @ ..] = dw.bytes else {
      return Err(E::from(MysqlError::InvalidOkBytes.into()));
    };
    let status = Status::try_from(u16::from_le_bytes([*a, *b]))?;
    let _warnings = u16::from_le_bytes([*c, *d]);
    *dw.bytes = rest;
    Ok(Self { affected_rows, status })
  }
}
