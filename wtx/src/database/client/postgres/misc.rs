use crate::{
  collections::{TryExtend, Vector},
  database::{
    Identifier,
    client::postgres::{
      PostgresRecord, PostgresRecords, PostgresStatement, PostgresStatements, Ty,
      msg_field::MsgField, postgres_column_info::PostgresColumnInfo,
    },
  },
  misc::Either,
  stream::BufStreamReader,
};
use core::ops::Range;

pub(crate) fn data_row<E>(
  begin_data: usize,
  read_buffer: &mut BufStreamReader,
  records_params: &mut Vector<(Range<usize>, Range<usize>)>,
  stmt: PostgresStatement<'_>,
  values_len: u16,
  values_params: &mut Vector<(bool, Range<usize>)>,
  values_params_offset: usize,
  cb: &mut impl FnMut(PostgresRecord<'_, E>) -> Result<(), E>,
) -> Result<(), E>
where
  E: From<crate::Error>,
{
  let net_buffer_range = begin_data..read_buffer.current_end_idx();
  let mut bytes = read_buffer.filled().get(net_buffer_range.clone()).unwrap_or_default();
  let rec_range_begin = read_buffer.antecedent_end_idx().wrapping_add(2).wrapping_sub(begin_data);
  let rec_range_end = read_buffer.current_end_idx().wrapping_sub(begin_data);
  bytes = bytes.get(rec_range_begin..rec_range_end).unwrap_or_default();
  let values_params_begin = values_params.len().wrapping_sub(values_params_offset);
  cb(PostgresRecord::parse(bytes, stmt, values_len, values_params)?)?;
  records_params.push((
    rec_range_begin..rec_range_end,
    values_params_begin..values_params.len().wrapping_sub(values_params_offset),
  ))?;
  Ok(())
}

pub(crate) const fn dummy_stmt_value() -> (PostgresColumnInfo, Ty) {
  (PostgresColumnInfo::new(Identifier::new(), Ty::Any), Ty::Any)
}

pub(crate) fn extend_records<'exec, B, E>(
  begin_data: usize,
  buffer: &mut B,
  read_buffer: &'exec mut BufStreamReader,
  records_params: &'exec mut Vector<(Range<usize>, Range<usize>)>,
  stmts: &'exec mut PostgresStatements,
  stmts_identifiers: impl IntoIterator<Item = Either<usize, u64>>,
  values_params: &'exec mut Vector<(bool, Range<usize>)>,
) -> crate::Result<()>
where
  B: TryExtend<[PostgresRecords<'exec, E>; 1]>,
{
  if B::IS_UNIT {
    return Ok(());
  }
  let mut rows_idx: usize = 0;
  let mut values_idx: usize = 0;
  for identifier in stmts_identifiers {
    let opt = match identifier {
      Either::Left(idx) => stmts.get_by_idx(idx),
      Either::Right(id) => stmts.get_by_stmt_cmd_id(id),
    };
    let Some(stmt) = opt else {
      return Err(crate::Error::ProgrammingError);
    };
    let local_rows_idx = rows_idx.wrapping_add(stmt.rows_len);
    let local_values_idx = values_idx.wrapping_add(stmt.columns_len.wrapping_mul(stmt.rows_len));
    let local_recp = records_params.get(rows_idx..local_rows_idx).unwrap_or_default();
    let local_valp = values_params.get(values_idx..local_values_idx).unwrap_or_default();
    rows_idx = local_rows_idx;
    values_idx = local_values_idx;
    buffer.try_extend([PostgresRecords::new(
      read_buffer.filled().get(begin_data..read_buffer.current_end_idx()).unwrap_or_default(),
      local_recp,
      stmt,
      local_valp,
    )])?;
  }
  Ok(())
}

pub(crate) fn row_description(
  columns_len: u16,
  rd: &mut &[u8],
  mut cb: impl FnMut(u16, PostgresColumnInfo) -> crate::Result<()>,
) -> crate::Result<()> {
  for idx in 0..columns_len {
    let (read, msg_field) = MsgField::parse(rd)?;
    let ty = Ty::from_arbitrary_u32(msg_field.type_oid);
    let pci = PostgresColumnInfo::new(msg_field.name.try_into()?, ty);
    cb(idx, pci)?;
    if let Some(elem @ [_not_empty, ..]) = rd.get(read..) {
      *rd = elem;
    } else {
      break;
    }
  }
  Ok(())
}
