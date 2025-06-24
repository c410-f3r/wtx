use crate::{
  database::client::mysql::{
    MysqlError,
    mysql_protocol::{
      MysqlProtocol, decode_wrapper_protocol::DecodeWrapperProtocol, lenenc::Lenenc,
    },
  },
  de::Decode,
  misc::Usize,
};

pub(crate) struct LenencContent<'bytes>(pub(crate) &'bytes [u8]);

impl<'de, DO, E> Decode<'de, MysqlProtocol<DO, E>> for LenencContent<'de>
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(aux: &mut (), dw: &mut DecodeWrapperProtocol<'de, '_, DO>) -> Result<Self, E> {
    let len = Lenenc::decode(aux, dw)?;
    let Some((lhs, rhs)) = dw.bytes.split_at_checked(*Usize::from(len.0)) else {
      return Err(E::from(MysqlError::InvalidLenencContentBytes.into()));
    };
    *dw.bytes = rhs;
    Ok(Self(lhs))
  }
}
