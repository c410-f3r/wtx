use crate::{
  collection::{TryExtend, Vector},
  database::{
    Identifier,
    client::postgres::{
      PostgresRecord, PostgresRecords, PostgresStatement, PostgresStatements, Ty,
      msg_field::MsgField, postgres_column_info::PostgresColumnInfo,
    },
  },
  misc::net::PartitionedFilledBuffer,
};
use core::ops::Range;

pub(crate) fn data_row<E>(
  begin: usize,
  begin_data: usize,
  net_buffer: &mut PartitionedFilledBuffer,
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
  let net_buffer_range = begin_data..net_buffer.current_end_idx();
  let mut bytes = net_buffer.all().get(net_buffer_range).unwrap_or_default();
  let record_range_begin = net_buffer.antecedent_end_idx().wrapping_sub(begin);
  let record_range_end = net_buffer.current_end_idx().wrapping_sub(begin_data);
  bytes = bytes.get(record_range_begin..record_range_end).unwrap_or_default();
  let values_params_begin = values_params.len().wrapping_sub(values_params_offset);
  cb(PostgresRecord::parse(bytes, stmt, values_len, values_params)?)?;
  records_params.push((
    record_range_begin..record_range_end,
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
  net_buffer: &'exec mut PartitionedFilledBuffer,
  records_params: &'exec mut Vector<(Range<usize>, Range<usize>)>,
  stmts: &'exec mut PostgresStatements,
  stmts_begin: usize,
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
  for idx in stmts_begin..stmts.len() {
    let Some(stmt) = stmts.get_by_idx(idx) else {
      return Err(crate::Error::ProgrammingError.into());
    };
    let local_rows_idx = rows_idx.wrapping_add(stmt.rows_len);
    let local_values_idx = stmt.columns_len.wrapping_mul(local_rows_idx);
    let local_rp = records_params.get(rows_idx..local_rows_idx).unwrap_or_default();
    let local_vp = values_params.get(values_idx..local_values_idx).unwrap_or_default();
    rows_idx = local_rows_idx;
    values_idx = local_values_idx;
    buffer.try_extend([PostgresRecords::new(
      net_buffer.all().get(begin_data..net_buffer.current_end_idx()).unwrap_or_default(),
      local_rp,
      stmt,
      local_vp,
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
    let ty = Ty::Custom(msg_field.type_oid);
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
