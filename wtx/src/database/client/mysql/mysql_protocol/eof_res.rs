use crate::{
  database::client::mysql::{
    MysqlError,
    mysql_protocol::{MysqlProtocol, decode_wrapper_protocol::DecodeWrapperProtocol},
    status::Status,
  },
  misc::Decode,
};

pub(crate) struct EofRes {
  pub(crate) status: Status,
}

impl<DO, E> Decode<'_, MysqlProtocol<DO, E>> for EofRes
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dw: &mut DecodeWrapperProtocol<'_, '_, DO>) -> Result<Self, E> {
    let [a, b, c, d, e] = dw.bytes else {
      return Err(E::from(MysqlError::InvalidEofBytes.into()));
    };
    if *a != 254 {
      return Err(E::from(MysqlError::InvalidEofBytes.into()));
    }
    let _warnings = u16::from_le_bytes([*b, *c]);
    let status = Status::try_from(u16::from_le_bytes([*d, *e]))?;
    Ok(Self { status })
  }
}
