use crate::{
  database::client::postgres::{Oid, PostgresError},
  misc::{bytes_pos1, from_utf8_basic},
};

#[derive(Debug)]
pub(crate) struct MsgField<'bytes> {
  pub(crate) name: &'bytes str,
  pub(crate) type_oid: Oid,
}

impl<'bytes> MsgField<'bytes> {
  #[inline]
  pub(crate) fn parse(value: &'bytes [u8]) -> crate::Result<(usize, Self)> {
    bytes_pos1(value, b'\0')
      .and_then(|idx| {
        let Some((
          name_bytes,
          &[_, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16, b17, b18, ..],
        )) = value.split_at_checked(idx)
        else {
          return None;
        };
        let name = from_utf8_basic(name_bytes).ok()?;
        let _table_oid = u32::from_be_bytes([b1, b2, b3, b4]);
        let _column_id = i16::from_be_bytes([b5, b6]);
        let type_oid = u32::from_be_bytes([b7, b8, b9, b10]);
        let _type_size = i16::from_be_bytes([b11, b12]);
        let _type_modifier = i32::from_be_bytes([b13, b14, b15, b16]);
        let _format = i16::from_be_bytes([b17, b18]);
        Some((name_bytes.len().wrapping_add(19), Self { name, type_oid }))
      })
      .ok_or_else(|| PostgresError::UnexpectedDatabaseMessageBytes.into())
  }
}
