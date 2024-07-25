use crate::database::{records::Records, Database, Decode, FromRecords, Record};

/// Seeks all rows that have an ID equal to `parent_record_id`.
#[inline]
pub fn seek_related_entities<'exec, D, T>(
  col_idx: &mut usize,
  curr_record_idx: &mut usize,
  parent_record_id: u64,
  parent_record_id_column_idx: usize,
  records: &D::Records<'exec>,
  mut entity_cb: impl FnMut(T) -> Result<(), D::Error>,
) -> Result<(), D::Error>
where
  D: Database,
  T: FromRecords<'exec, D>,
  for<'de> u64: Decode<'de, D>,
{
  let initial_col_idx = *col_idx;
  loop {
    let Some(curr_record) = records.get(*curr_record_idx) else {
      break;
    };
    let local_parent_record_id = curr_record.decode::<_, u64>(parent_record_id_column_idx)?;
    if local_parent_record_id == parent_record_id {
      entity_cb(T::from_records(col_idx, &curr_record, curr_record_idx, records)?)?;
      *col_idx = initial_col_idx;
      *curr_record_idx = curr_record_idx.wrapping_add(1);
    } else {
      break;
    }
  }
  Ok(())
}
