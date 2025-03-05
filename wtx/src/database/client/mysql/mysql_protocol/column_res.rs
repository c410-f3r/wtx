use crate::{
  database::{
    Identifier,
    client::mysql::{
      MysqlError,
      mysql_protocol::{
        MysqlProtocol, decode_wrapper_protocol::DecodeWrapperProtocol, lenenc::Lenenc,
        lenenc_content::LenencContent,
      },
      ty::Ty,
    },
  },
  misc::Decode,
};

#[derive(Debug)]
pub(crate) struct ColumnRes {
  pub(crate) alias: Identifier,
  pub(crate) flags: u16,
  pub(crate) name: Identifier,
  pub(crate) ty: Ty,
}

impl<DO, E> Decode<'_, MysqlProtocol<DO, E>> for ColumnRes
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(aux: &mut (), dw: &mut DecodeWrapperProtocol<'_, '_, DO>) -> Result<Self, E> {
    let _catalog = LenencContent::decode(aux, dw)?.0;
    let _schema = LenencContent::decode(aux, dw)?.0;
    let _table_alias = LenencContent::decode(aux, dw)?.0;
    let _table = LenencContent::decode(aux, dw)?.0;
    let column_alias = LenencContent::decode(aux, dw)?.0;
    let column_name = LenencContent::decode(aux, dw)?.0;
    let _next_len = Lenenc::decode(aux, dw)?;
    let [a, b, c, d, e, f, g, h, i, j, ..] = dw.bytes else {
      return Err(E::from(MysqlError::InvalidColumnBytes.into()));
    };
    let _collation = u16::from_le_bytes([*a, *b]);
    let _max_size = u32::from_le_bytes([*c, *d, *e, *f]);
    let type_id = *g;
    let flags = u16::from_le_bytes([*h, *i]);
    let _decimals = *j;
    Ok(Self {
      alias: column_alias.try_into()?,
      flags,
      name: column_name.try_into()?,
      ty: Ty::try_from(type_id)?,
    })
  }
}
