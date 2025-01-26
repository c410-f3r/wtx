use crate::database::{records::Records, Database, Decode, FromRecords, Record};

/// Seeks all rows that have an ID equal to `parent_record_id`.
#[inline]
pub fn seek_related_entities<'exec, D, ID, T>(
  (curr_field_idx, curr_record_idx): (&mut usize, &mut usize),
  (parent_record_id, parent_record_id_column_idx): (ID, usize),
  records: &D::Records<'exec>,
  mut entity_cb: impl FnMut(T) -> Result<(), D::Error>,
) -> Result<(), D::Error>
where
  D: Database,
  T: FromRecords<'exec, D>,
  ID: Decode<'exec, D> + PartialEq,
{
  let initial_col_idx = *curr_field_idx;
  loop {
    let Some(curr_record) = records.get(*curr_record_idx) else {
      break;
    };
    let local_parent_record_id = curr_record.decode::<_, ID>(parent_record_id_column_idx)?;
    if local_parent_record_id == parent_record_id {
      entity_cb(T::from_records((curr_field_idx, &curr_record, curr_record_idx), records)?)?;
      *curr_field_idx = initial_col_idx;
      *curr_record_idx = curr_record_idx.wrapping_add(1);
    } else {
      break;
    }
  }
  Ok(())
}
