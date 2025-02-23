use crate::{
  database::{
    RecordValues,
    client::mysql::{
      EncodeWrapper, Mysql, MysqlStatement,
      flags::Flags,
      mysql_protocol::{MysqlProtocol, encode_wrapper_protocol::EncodeWrapperProtocol},
    },
  },
  misc::Encode,
};

#[derive(Debug)]
pub(crate) struct StmtExecuteReq<'any, 'stmts, RV> {
  pub(crate) rv: RV,
  pub(crate) stmt: &'any MysqlStatement<'stmts>,
}

impl<'any, 'stmts, DO, E, RV> Encode<MysqlProtocol<DO, E>> for StmtExecuteReq<'any, 'stmts, RV>
where
  E: From<crate::Error>,
  RV: RecordValues<Mysql<E>>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapperProtocol<'_>) -> Result<(), E> {
    let _ = ew.enc_buffer.extend_from_copyable_slices([
      &[23][..],
      &self.stmt._aux().to_le_bytes(),
      &[0],
      &1_u32.to_le_bytes(),
    ])?;
    if self.stmt._tys_len() > 0 {
      let prev_len = ew.enc_buffer.len();
      let mut idx: usize = 0;
      self.rv.is_null(|is_null| {
        let bit_offset = idx % 8;
        let byte_index = prev_len.wrapping_add(idx / 8);
        if bit_offset == 0 {
          ew.enc_buffer.push(0)?;
        }
        idx = idx.wrapping_add(1);
        if let Some(elem) = ew.enc_buffer.get_mut(byte_index) {
          *elem |= u8::from(is_null) << bit_offset;
        }
        Ok(())
      })?;
      for ty in self.stmt._tys() {
        let unsigned = u16::from(Flags::Unsigned);
        let value = if ty.flags & unsigned == unsigned { 128 } else { 0 };
        ew.enc_buffer.extend_from_copyable_slice(&[u8::from(ty.ty), value])?;
      }
      let _ = self.rv.encode_values(
        &mut (),
        &mut EncodeWrapper::new(ew.enc_buffer),
        |_, _| 0,
        |_, _, _, _| 0,
      )?;
    }
    Ok(())
  }
}
