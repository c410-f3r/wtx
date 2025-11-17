use crate::{
  database::{
    RecordValues,
    client::mysql::{
      EncodeWrapper, Mysql, TyParams,
      command::Command,
      flag::Flag,
      mysql_column_info::MysqlColumnInfo,
      protocol::{Protocol, encode_wrapper_protocol::EncodeWrapperProtocol},
    },
  },
  de::Encode,
};

#[derive(Debug)]
pub(crate) struct StmtExecuteReq<'any, RV> {
  pub(crate) rv: RV,
  pub(crate) stmt_id: u32,
  pub(crate) tys: &'any [(MysqlColumnInfo, TyParams)],
}

impl<'any, E, RV> Encode<Protocol<(), E>> for StmtExecuteReq<'any, RV>
where
  E: From<crate::Error>,
  RV: RecordValues<Mysql<E>>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapperProtocol<'_>) -> Result<(), E> {
    let _ = ew.encode_buffer.extend_from_copyable_slices([
      &[Command::ComStmtExecute.into()][..],
      &self.stmt_id.to_le_bytes(),
      &[0, 1, 0, 0, 0],
    ])?;
    if !self.tys.is_empty() {
      let prev_len = ew.encode_buffer.len();
      let mut idx: usize = 0;
      self.rv.walk(|is_null, _| {
        let bit_offset = idx % 8;
        let byte_index = prev_len.wrapping_add(idx / 8);
        if bit_offset == 0 {
          ew.encode_buffer.push(0)?;
        }
        idx = idx.wrapping_add(1);
        if let Some(elem) = ew.encode_buffer.get_mut(byte_index) {
          *elem |= u8::from(is_null) << bit_offset;
        }
        Ok(())
      })?;
      ew.encode_buffer.push(1)?;
      for (_, ty) in self.tys {
        let unsigned = u16::from(Flag::Unsigned);
        let value = if ty.flags() & unsigned == unsigned { 128 } else { 0 };
        ew.encode_buffer.extend_from_copyable_slice(&[u8::from(ty.ty()), value])?;
      }
      let _ = self.rv.encode_values(
        &mut (),
        &mut EncodeWrapper::new(ew.encode_buffer),
        |_, _| 0,
        |_, _, _, _| 0,
      )?;
    }
    Ok(())
  }
}
