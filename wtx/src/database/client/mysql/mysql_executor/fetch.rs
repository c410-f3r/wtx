use crate::{
  database::{
    RecordValues, StmtCmd,
    client::mysql::{
      ExecutorBuffer, Mysql, MysqlError, MysqlExecutor, MysqlRecord, MysqlStatement,
      MysqlStatements,
      column::Column,
      misc::{decode, fetch_msg, fetch_protocol, send_packet},
      mysql_protocol::{
        binary_row_res::BinaryRowRes, lenenc::Lenenc, ok_res::OkRes,
        stmt_execute_req::StmtExecuteReq, text_row_res::TextRowRes,
      },
      status::Status,
    },
  },
  misc::{LeaseMut, Stream, Usize, Vector, partitioned_filled_buffer::PartitionedFilledBuffer},
};
use core::ops::Range;

impl<E, EB, S> MysqlExecutor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  #[inline]
  pub(crate) async fn fetch_cmd<const IS_BIN: bool, const IS_SINGLE: bool>(
    capabilities: u64,
    net_buffer: &mut PartitionedFilledBuffer,
    records_params: &mut Vector<(Range<usize>, Range<usize>)>,
    sequence_id: &mut u8,
    stmt: &MysqlStatement<'_>,
    stream: &mut S,
    values_params: &mut Vector<(bool, Range<usize>)>,
    mut cb_end: impl FnMut(u64) -> Result<(), E>,
    mut cb_rslt: impl FnMut(MysqlRecord<'_, E>) -> Result<(), E>,
  ) -> Result<(), E> {
    let smre = u16::from(Status::ServerMoreResultsExists);
    let mut affected_rows: u64 = 0;
    let mut end: usize = 0;
    let mut has_at_least_one_record = false;
    loop {
      let total0 = fetch_msg(capabilities, net_buffer, sequence_id, stream).await?;
      end = end.wrapping_add(total0);
      let mut local_rest = net_buffer._current();
      let local_rest_first = local_rest.first().copied();
      if local_rest_first == Some(0) || local_rest_first == Some(255) {
        let res: OkRes = decode(&mut local_rest, ())?;
        if u16::from(res.statuses) & smre == smre {
          continue;
        }
        affected_rows = affected_rows.wrapping_add(res.affected_rows);
        cb_end(affected_rows)?;
        return Ok(());
      }

      let columns_lenenc: Lenenc = decode(&mut local_rest, ())?;
      let columns = Usize::try_from(columns_lenenc.0)?.into_usize();

      for _ in 0..columns {
        let (res, total1) = fetch_protocol(capabilities, net_buffer, sequence_id, stream).await?;
        end = end.wrapping_add(total1);
        let _column = Column::from_column_res(&res);
      }

      loop {
        let record_begin = end;
        let total2 = fetch_msg(capabilities, net_buffer, sequence_id, stream).await?;
        end = end.wrapping_add(total2);
        let mut current = net_buffer._current();
        if current.first() == Some(&254) && current.len() < 9 {
          let res: OkRes = decode(&mut current, ())?;
          cb_end(0)?;
          if u16::from(res.statuses) & smre == smre {
            break;
          }
          return Ok(());
        }
        if IS_SINGLE {
          if has_at_least_one_record {
            return Err(E::from(MysqlError::NonSingleFetch.into()));
          } else {
            has_at_least_one_record = true;
          }
        }
        let val_begin = values_params.len();
        if IS_BIN {
          let res: BinaryRowRes<'_> = decode(&mut current, (stmt, &mut *values_params))?;
          let rec = MysqlRecord::new(
            res.0,
            stmt.clone(),
            values_params.get(val_begin..).unwrap_or_default(),
          );
          cb_rslt(rec)?;
        } else {
          let _row: TextRowRes = decode(&mut current, (columns, &mut *values_params))?;
        }
        records_params.push((record_begin.wrapping_add(4)..end, val_begin..values_params.len()))?;
      }
    }
  }

  #[inline]
  pub(crate) async fn write_send_await_stmt<'stmts, RV, SC, const IS_SINGLE: bool>(
    (capabilities, sequence_id): (&mut u64, &mut u8),
    encode_buffer: &mut Vector<u8>,
    net_buffer: &mut PartitionedFilledBuffer,
    records_params: &mut Vector<(Range<usize>, Range<usize>)>,
    rv: RV,
    sc: SC,
    stmts: &'stmts mut MysqlStatements,
    stream: &mut S,
    values_params: &mut Vector<(bool, Range<usize>)>,
    cb_end: impl FnMut(u64) -> Result<(), E>,
    cb_rslt: impl FnMut(MysqlRecord<'_, E>) -> Result<(), E>,
  ) -> Result<(usize, MysqlStatement<'stmts>), E>
  where
    RV: RecordValues<Mysql<E>>,
    SC: StmtCmd,
  {
    let (_, _, stmt) = Self::write_send_await_stmt_prot(
      (capabilities, sequence_id),
      encode_buffer,
      net_buffer,
      sc,
      stmts,
      stream,
      |stmt_mut| {
        if *stmt_mut.tys_len == 0 && rv.len() > 0 {
          let len = rv.len().min(stmt_mut.values.len());
          let mut values = stmt_mut.values.iter_mut().take(len);
          rv.walk(|_, ty_opt| {
            if let (Some(ty), Some(value)) = (ty_opt, values.next()) {
              value.1 = ty;
            }
            Ok(())
          })?;
          *stmt_mut.tys_len = len;
        }
        Ok(())
      },
    )
    .await?;
    send_packet(
      (capabilities, sequence_id),
      encode_buffer,
      StmtExecuteReq { rv, stmt: &stmt },
      stream,
    )
    .await?;
    let start = net_buffer._current_end_idx();
    Self::fetch_cmd::<true, IS_SINGLE>(
      *capabilities,
      net_buffer,
      records_params,
      sequence_id,
      &stmt,
      stream,
      values_params,
      cb_end,
      cb_rslt,
    )
    .await?;
    Ok((start, stmt))
  }
}
