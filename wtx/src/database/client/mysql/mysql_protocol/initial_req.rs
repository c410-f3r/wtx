use crate::{
  database::client::mysql::{
    capability::Capability,
    collation::Collation,
    mysql_protocol::{MysqlProtocol, encode_wrapper_protocol::EncodeWrapperProtocol},
  },
  misc::Encode,
};

#[derive(Debug)]
pub(crate) struct InitialReq {
  pub(crate) collation: Collation,
  pub(crate) max_packet_size: u32,
}

impl<E> Encode<MysqlProtocol<(), E>> for InitialReq
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapperProtocol<'_>) -> Result<(), E> {
    let capability_lhs = (*ew.capabilities >> 32) as u32;
    let capability_rhs = *ew.capabilities as u32;
    let _ = ew.enc_buffer.extend_from_copyable_slices([
      capability_rhs.to_le_bytes().as_slice(),
      self.max_packet_size.to_le_bytes().as_slice(),
      &[self.collation.into()],
      &[0; 19],
    ])?;
    let mysql_n = u64::from(Capability::Mysql);
    ew.enc_buffer.extend_from_copyable_slice(&if *ew.capabilities & mysql_n == mysql_n {
      [0; 4]
    } else {
      capability_lhs.to_le_bytes()
    })?;
    Ok(())
  }
}
