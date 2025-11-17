use crate::{
  collection::{TryExtend, Vector},
  database::{
    StmtCmd,
    client::{
      mysql::{
        ExecutorBuffer, MysqlExecutor, MysqlRecord, MysqlRecords, MysqlStatementMut,
        MysqlStatements,
        misc::{decode, dummy_stmt_value, fetch_msg, fetch_protocol},
        mysql_column_info::MysqlColumnInfo,
        protocol::{
          binary_row_res::BinaryRowRes, lenenc::Lenenc, ok_res::OkRes, text_row_res::TextRowRes,
        },
        status::Status,
      },
      rdbms::{statement::StatementMut, statements_misc::StatementsMisc},
    },
  },
  misc::{LeaseMut, Usize, net::PartitionedFilledBuffer, timestamp_nanos_str},
  stream::Stream,
};
use core::ops::{ControlFlow, Range};

impl<E, EB, S> MysqlExecutor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  pub(crate) async fn fetch_bin_cmd(
    capabilities: u64,
    net_buffer: &mut PartitionedFilledBuffer,
    records_params: &mut Vector<(Range<usize>, Range<usize>)>,
    sequence_id: &mut u8,
    stmt_mut: &mut MysqlStatementMut<'_>,
    stream: &mut S,
    values_params: &mut Vector<(bool, Range<usize>)>,
    mut cb: impl FnMut(MysqlRecord<'_, E>) -> Result<(), E>,
  ) -> Result<(), E> {
    let mut end = 0;
    let mut rows = 0;
    loop {
      let cf = Self::fetch_init(capabilities, &mut end, net_buffer, sequence_id, stream).await?;
      let columns = match cf {
        ControlFlow::Continue(None) => continue,
        ControlFlow::Continue(Some(el)) => el,
        ControlFlow::Break(()) => break,
      };
      Self::fetch_columns(
        capabilities,
        columns,
        &mut end,
        net_buffer,
        sequence_id,
        stream,
        |_, _| Ok(()),
      )
      .await?;
      let should_stop_loop = Self::fetch_rows(
        capabilities,
        &mut end,
        net_buffer,
        records_params,
        &mut rows,
        sequence_id,
        stream,
        |mut current| {
          let val_begin = values_params.len();
          let stmt = stmt_mut.stmt();
          let res: BinaryRowRes<'_> = decode(&mut current, (&stmt, &mut *values_params))?;
          cb(MysqlRecord::new(
            res.0,
            stmt_mut.stmt(),
            values_params.get(val_begin..).unwrap_or_default(),
          ))?;
          Ok(val_begin..values_params.len())
        },
      )
      .await?;
      if should_stop_loop {
        break;
      }
    }
    *stmt_mut.rows_len = *Usize::from(rows);
    Ok(())
  }

  pub(crate) async fn fetch_text_cmd<'exec, B>(
    buffer: &mut B,
    capabilities: u64,
    net_buffer: &'exec mut PartitionedFilledBuffer,
    records_params: &'exec mut Vector<(Range<usize>, Range<usize>)>,
    sequence_id: &mut u8,
    stmts: &'exec mut MysqlStatements,
    stream: &mut S,
    values_params: &'exec mut Vector<(bool, Range<usize>)>,
    mut cb: impl FnMut(MysqlRecord<'_, E>) -> Result<(), E>,
  ) -> Result<(), E>
  where
    B: TryExtend<[MysqlRecords<'exec, E>; 1]>,
  {
    let begin_data = net_buffer.current_end_idx();
    let stmts_begin = stmts.len();
    let mut end = 0;
    let mut rows = 0;
    let mut values_params_offset = 0;

    loop {
      let columns_len =
        match Self::fetch_init(capabilities, &mut end, net_buffer, sequence_id, stream).await? {
          ControlFlow::Continue(None) => continue,
          ControlFlow::Continue(Some(el)) => el,
          ControlFlow::Break(()) => {
            break;
          }
        };
      let timestamp_nanos_str = timestamp_nanos_str()?;
      let stmt_cmd_id = timestamp_nanos_str.as_str().hash(stmts.hasher_mut());
      let mut builder = stmts
        .builder((), {
          async fn fun(_: &mut (), _: StatementsMisc<u32>) -> crate::Result<()> {
            Ok(())
          }
          fun
        })
        .await?;
      if !B::IS_UNIT {
        let _ = builder.expand(columns_len, dummy_stmt_value())?;
      }
      let inserted_elements = builder.inserted_elements();
      Self::fetch_columns(
        capabilities,
        columns_len,
        &mut end,
        net_buffer,
        sequence_id,
        stream,
        |column_idx, mci| {
          if let (false, Some(elem)) = (B::IS_UNIT, inserted_elements.get_mut(column_idx)) {
            elem.0 = mci;
          }
          Ok(())
        },
      )
      .await?;
      let (stmt_columns_len, stmt_rows_len, stmt_tys_len) = (&mut 0, &mut 0, &mut 0);
      let stmt_mut = if B::IS_UNIT {
        StatementMut::new(0, stmt_columns_len, stmt_rows_len, stmt_tys_len, &mut [])
      } else {
        let sm = StatementsMisc::new(0, columns_len.into(), 0, 0);
        let stmt_idx = builder.build(stmt_cmd_id, sm)?;
        let Some(stmt_mut) = stmts.get_by_idx_mut(stmt_idx) else {
          return Err(crate::Error::ProgrammingError.into());
        };
        stmt_mut
      };
      let should_stop_loop = Self::fetch_rows(
        capabilities,
        &mut end,
        net_buffer,
        records_params,
        &mut rows,
        sequence_id,
        stream,
        |mut current| {
          let val_begin = values_params.len();
          let res: TextRowRes<'_> = decode(&mut current, (columns_len, &mut *values_params))?;
          let val_end = values_params.len();
          cb(MysqlRecord::new(
            res.0,
            stmt_mut.stmt(),
            values_params.get(val_begin..val_end).unwrap_or_default(),
          ))?;
          Ok(
            val_begin.wrapping_sub(values_params_offset)
              ..val_end.wrapping_sub(values_params_offset),
          )
        },
      )
      .await?;
      *stmt_mut.rows_len = *Usize::from(rows);
      values_params_offset = values_params.len();
      if should_stop_loop {
        break;
      }
    }

    if !B::IS_UNIT {
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
        buffer.try_extend([MysqlRecords::new(
          net_buffer.all().get(begin_data..net_buffer.current_end_idx()).unwrap_or_default(),
          local_rp,
          stmt,
          local_vp,
        )])?;
      }
    }
    Ok(())
  }

  async fn fetch_columns(
    capabilities: u64,
    columns: usize,
    end: &mut usize,
    net_buffer: &mut PartitionedFilledBuffer,
    sequence_id: &mut u8,
    stream: &mut S,
    mut cb: impl FnMut(usize, MysqlColumnInfo) -> Result<(), E>,
  ) -> Result<(), E> {
    for idx in 0..columns {
      let (res, total1) = fetch_protocol(capabilities, net_buffer, sequence_id, stream).await?;
      *end = end.wrapping_add(total1);
      cb(idx, MysqlColumnInfo::from_column_res(&res))?;
    }
    Ok(())
  }

  async fn fetch_init(
    capabilities: u64,
    end: &mut usize,
    net_buffer: &mut PartitionedFilledBuffer,
    sequence_id: &mut u8,
    stream: &mut S,
  ) -> Result<ControlFlow<(), Option<usize>>, E> {
    let total0 = fetch_msg(capabilities, net_buffer, sequence_id, stream).await?;
    *end = end.wrapping_add(total0);
    let mut local_rest = net_buffer.current();
    let local_rest_first = local_rest.first().copied();
    if local_rest_first == Some(0) || local_rest_first == Some(255) {
      let res: OkRes = decode(&mut local_rest, ())?;
      let smre = u16::from(Status::ServerMoreResultsExists);
      if res.statuses & smre == smre {
        return Ok(ControlFlow::Continue(None));
      }
      return Ok(ControlFlow::Break(()));
    }
    let columns_lenenc: Lenenc = decode(&mut local_rest, ())?;
    let columns = Usize::from(columns_lenenc.0).into_usize();
    Ok(ControlFlow::Continue(Some(columns)))
  }

  /// If `true` is returned, the outer loop must be stopped.
  async fn fetch_rows(
    capabilities: u64,
    end: &mut usize,
    net_buffer: &mut PartitionedFilledBuffer,
    records_params: &mut Vector<(Range<usize>, Range<usize>)>,
    rows: &mut u32,
    sequence_id: &mut u8,
    stream: &mut S,
    mut cb: impl FnMut(&[u8]) -> Result<Range<usize>, E>,
  ) -> Result<bool, E> {
    *rows = 0;
    loop {
      let record_begin = *end;
      let total2 = fetch_msg(capabilities, net_buffer, sequence_id, stream).await?;
      *end = end.wrapping_add(total2);
      let mut current = net_buffer.current();
      if current.first() == Some(&254) && current.len() < 9 {
        let res: OkRes = decode(&mut current, ())?;
        let smre = u16::from(Status::ServerMoreResultsExists);
        if res.statuses & smre == smre {
          return Ok(false);
        }
        return Ok(true);
      }
      *rows = rows.wrapping_add(1);
      records_params.push((record_begin.wrapping_add(4)..*end, cb(current)?))?;
    }
  }
}
