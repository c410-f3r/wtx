use crate::database::{Database, FromRecords, FromRecordsParams, Record, records::Records};
use core::ops::ControlFlow;

/// Seeks all rows that have an ID equal to `parent_record_id`.
#[inline]
pub fn seek_related_entities<'exec, D, T>(
  curr_params: &mut FromRecordsParams<D::Record<'exec>>,
  (parent_record_id, parent_record_id_field_idx): (T::IdTy, usize),
  records: &D::Records<'exec>,
  mut entity_cb: impl FnMut(T) -> Result<(), D::Error>,
) -> Result<(), D::Error>
where
  D: Database,
  T: FromRecords<'exec, D>,
{
  let initial_field_idx = curr_params.curr_field_idx;
  let prev_record_idx = curr_params.curr_record_idx;
  let mut next_field_idx = curr_params.curr_field_idx;
  let local_id_field_idx = T::ID_IDX.map(|el| el.wrapping_add(curr_params.curr_field_idx));
  'block: {
    let Some(curr_record) = records.get(curr_params.curr_record_idx) else {
      break 'block;
    };
    let initial_local_id_opt = if let Some(idx) = local_id_field_idx {
      Some(curr_record.decode::<_, T::IdTy>(idx)?)
    } else {
      None
    };
    curr_params.curr_record = curr_record;
    if seek(
      curr_params,
      (&parent_record_id, parent_record_id_field_idx),
      (initial_field_idx, &mut next_field_idx),
      records,
      &mut entity_cb,
    )?
    .is_break()
    {
      break 'block;
    }
    let mut prev_local_id_opt = initial_local_id_opt;
    loop {
      let Some(curr_record) = records.get(curr_params.curr_record_idx) else {
        break;
      };
      if let (Some(idx), Some(initial_local_id)) = (local_id_field_idx, initial_local_id_opt) {
        let curr_local_id_opt = curr_record.decode_opt::<_, T::IdTy>(idx)?;
        let Some(curr_local_id) = curr_local_id_opt else {
          curr_params.inc_record_idx();
          continue;
        };
        if prev_local_id_opt == Some(curr_local_id) {
          curr_params.inc_record_idx();
          continue;
        }
        if curr_local_id == initial_local_id {
          break;
        }
        prev_local_id_opt = Some(curr_local_id);
      }
      curr_params.curr_record = curr_record;
      if seek(
        curr_params,
        (&parent_record_id, parent_record_id_field_idx),
        (initial_field_idx, &mut next_field_idx),
        records,
        &mut entity_cb,
      )?
      .is_break()
      {
        break;
      }
    }
  }
  curr_params.inc_consumed_records(curr_params.curr_record_idx.wrapping_sub(prev_record_idx));
  curr_params.curr_field_idx = next_field_idx;
  curr_params.curr_record_idx = prev_record_idx;
  Ok(())
}

#[inline]
fn seek<'exec, D, T>(
  curr_params: &mut FromRecordsParams<D::Record<'exec>>,
  (parent_record_id, parent_record_id_field_idx): (&T::IdTy, usize),
  (initial_field_idx, next_field_idx): (usize, &mut usize),
  records: &D::Records<'exec>,
  entity_cb: &mut impl FnMut(T) -> Result<(), D::Error>,
) -> Result<ControlFlow<()>, D::Error>
where
  D: Database,
  T: FromRecords<'exec, D>,
{
  let local_id: T::IdTy = curr_params.curr_record.decode(parent_record_id_field_idx)?;
  if local_id == *parent_record_id {
    entity_cb(T::from_records(curr_params, records)?)?;
    *next_field_idx = curr_params.curr_field_idx;
    curr_params.curr_field_idx = initial_field_idx;
    curr_params.inc_record_idx();
  } else {
    return Ok(ControlFlow::Break(()));
  }
  Ok(ControlFlow::Continue(()))
}
