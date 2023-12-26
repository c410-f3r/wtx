use crate::{database::client::postgres::Oid, misc::from_utf8_basic_rslt};

#[derive(Debug)]
pub(crate) struct MsgField<'bytes> {
  pub(crate) name: &'bytes str,
  pub(crate) type_oid: Oid,
}

impl<'bytes> MsgField<'bytes> {
  pub(crate) fn parse(value: &'bytes [u8]) -> crate::Result<(usize, Self)> {
    let (name_bytes, rest_bytes) = value.split_at(
      value
        .iter()
        .position(|el| *el == b'\0')
        .ok_or(crate::Error::UnexpectedDatabaseMessageBytes)?,
    );
    let &[_, a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, ..] = rest_bytes else {
      return Err(crate::Error::UnexpectedDatabaseMessageBytes);
    };
    let name = from_utf8_basic_rslt(name_bytes)?;
    let _table_oid = u32::from_be_bytes([a, b, c, d]);
    let _column_id = i16::from_be_bytes([e, f]);
    let type_oid = u32::from_be_bytes([g, h, i, j]);
    let _type_size = i16::from_be_bytes([k, l]);
    let _type_modifier = i32::from_be_bytes([m, n, o, p]);
    let _format = i16::from_be_bytes([q, r]);
    Ok((name_bytes.len().wrapping_add(19), Self { name, type_oid }))
  }
}
