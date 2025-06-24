use crate::{
  database::client::mysql::{
    MysqlError,
    mysql_protocol::{MysqlProtocol, decode_wrapper_protocol::DecodeWrapperProtocol},
  },
  de::Decode,
};

#[derive(Debug)]
pub(crate) struct PrepareRes {
  pub(crate) columns: u16,
  pub(crate) params: u16,
  pub(crate) statement_id: u32,
}

impl<'de, DO, E> Decode<'de, MysqlProtocol<DO, E>> for PrepareRes
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dw: &mut DecodeWrapperProtocol<'de, '_, DO>) -> Result<Self, E> {
    let [a, b, c, d, e, f, g, h, i, _, k, l, ..] = dw.bytes else {
      return Err(E::from(MysqlError::InvalidPrepareBytes.into()));
    };
    if *a != 0 {
      return Err(E::from(MysqlError::InvalidPrepareBytes.into()));
    }
    let statement_id = u32::from_le_bytes([*b, *c, *d, *e]);
    let columns = u16::from_le_bytes([*f, *g]);
    let params = u16::from_le_bytes([*h, *i]);
    let _warnings = u16::from_le_bytes([*k, *l]);
    Ok(Self { columns, params, statement_id })
  }
}
