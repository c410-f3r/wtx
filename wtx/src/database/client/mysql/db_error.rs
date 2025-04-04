use crate::{
  database::client::mysql::{
    MysqlError,
    capability::Capability,
    mysql_protocol::{MysqlProtocol, decode_wrapper_protocol::DecodeWrapperProtocol},
  },
  misc::{ArrayString, Decode, from_utf8_basic},
};
use alloc::string::String;

/// Error returned by the server
#[derive(Debug)]
pub struct DbError {
  /// Code
  pub error_code: u16,
  /// Message
  pub error_message: String,
  /// State
  pub sql_state: Option<ArrayString<5>>,
}

impl<E> Decode<'_, MysqlProtocol<u64, E>> for DbError
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dw: &mut DecodeWrapperProtocol<'_, '_, u64>) -> Result<Self, E> {
    let [255, a, b, rest0 @ ..] = dw.bytes else {
      return Err(E::from(MysqlError::InvalidErrPacketResBytes.into()));
    };
    let mut bytes = rest0;
    let error_code = u16::from_le_bytes([*a, *b]);
    let mut sql_state = None;
    let protocol_41_n = u64::from(Capability::Protocol41);
    if dw.other & protocol_41_n == protocol_41_n {
      if let [b'#', c, d, e, f, g, rest1 @ ..] = rest0 {
        let array = ArrayString::from_parts([*c, *d, *e, *f, *g], 5)?;
        sql_state = Some(array);
        bytes = rest1;
      }
    }
    let mut error_message = String::new();
    error_message.push_str(from_utf8_basic(bytes).map_err(crate::Error::from)?);
    Ok(Self { error_code, error_message, sql_state })
  }
}
