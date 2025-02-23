use crate::{
  database::client::mysql::mysql_protocol::{
    MysqlProtocol, decode_wrapper_protocol::DecodeWrapperProtocol, lenenc::Lenenc,
  },
  misc::{Decode, Usize},
};

pub(crate) struct LenencBytes<'bytes>(pub(crate) &'bytes [u8]);

impl<'de, DO, E> Decode<'de, MysqlProtocol<DO, E>> for LenencBytes<'de>
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(aux: &mut (), dw: &mut DecodeWrapperProtocol<'de, '_, DO>) -> Result<Self, E> {
    let len = Lenenc::decode(aux, dw)?;
    let Some((lhs, rhs)) = dw.bytes.split_at_checked(*Usize::try_from(len.0)?) else {
      panic!();
    };
    *dw.bytes = rhs;
    Ok(Self(lhs))
  }
}
