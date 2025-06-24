use crate::{
  database::client::mysql::{
    MysqlError,
    mysql_protocol::{
      MysqlProtocol, decode_wrapper_protocol::DecodeWrapperProtocol, lenenc::Lenenc,
    },
  },
  de::Decode,
};

#[derive(Debug)]
pub(crate) struct OkRes {
  pub(crate) affected_rows: u64,
  pub(crate) statuses: u16,
}

impl<DO, E> Decode<'_, MysqlProtocol<DO, E>> for OkRes
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dw: &mut DecodeWrapperProtocol<'_, '_, DO>) -> Result<Self, E> {
    let [first, rest0 @ ..] = dw.bytes else {
      return Err(E::from(MysqlError::InvalidOkBytes.into()));
    };
    if *first != 0 && *first != 254 {
      return Err(E::from(MysqlError::InvalidOkBytes.into()));
    }
    *dw.bytes = rest0;
    let affected_rows = Lenenc::decode(&mut (), dw)?.0;
    let _last_insert_id = Lenenc::decode(&mut (), dw)?.0;
    let [a, b, c, d, rest1 @ ..] = dw.bytes else {
      return Err(E::from(MysqlError::InvalidOkBytes.into()));
    };
    let statuses = u16::from_le_bytes([*a, *b]);
    let _warnings = u16::from_le_bytes([*c, *d]);
    *dw.bytes = rest1;
    Ok(Self { affected_rows, statuses })
  }
}
