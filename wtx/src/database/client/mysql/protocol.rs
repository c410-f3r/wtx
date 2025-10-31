pub(crate) mod auth_switch_req;
pub(crate) mod auth_switch_res;
pub(crate) mod binary_row_res;
pub(crate) mod column_res;
pub(crate) mod decode_wrapper_protocol;
pub(crate) mod encode_wrapper_protocol;
pub(crate) mod handshake_req;
pub(crate) mod handshake_res;
pub(crate) mod initial_req;
pub(crate) mod lenenc;
pub(crate) mod lenenc_content;
pub(crate) mod ok_res;
pub(crate) mod packet_req;
pub(crate) mod prepare_req;
pub(crate) mod prepare_res;
pub(crate) mod query_req;
pub(crate) mod stmt_close_req;
pub(crate) mod stmt_execute_req;
pub(crate) mod text_row_res;

use crate::{
  database::client::mysql::protocol::{
    decode_wrapper_protocol::DecodeWrapperProtocol, encode_wrapper_protocol::EncodeWrapperProtocol,
  },
  de::{DEController, Encode},
};
use core::marker::PhantomData;

pub(crate) struct Protocol<DO, E>(PhantomData<(DO, E)>);

impl<DO, E> DEController for Protocol<DO, E>
where
  E: From<crate::Error>,
{
  type Aux = ();
  type DecodeWrapper<'inner, 'outer, 'rem>
    = DecodeWrapperProtocol<'inner, 'outer, DO>
  where
    'inner: 'outer;
  type Error = E;
  type EncodeWrapper<'inner, 'outer, 'rem>
    = EncodeWrapperProtocol<'inner>
  where
    'inner: 'outer;
}

impl<DO, E> Encode<Protocol<DO, E>> for &[u8]
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapperProtocol<'_>) -> Result<(), E> {
    ew.encode_buffer.extend_from_copyable_slice(self)?;
    Ok(())
  }
}
