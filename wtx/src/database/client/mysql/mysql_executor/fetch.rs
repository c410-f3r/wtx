use core::{marker::PhantomData, ops::Range};

use crate::{
  database::{
    RecordValues, StmtCmd,
    client::mysql::{
      ExecutorBuffer, Mysql, MysqlError, MysqlExecutor, MysqlRecord, MysqlStatement,
      MysqlStatements,
      column::Column,
      misc::{decode, fetch_msg, fetch_protocol, send_packet},
      mysql_protocol::{
        binary_row_res::BinaryRowRes, column_res::ColumnRes, eof_res::EofRes, lenenc::Lenenc,
        ok_res::OkRes, stmt_execute_req::StmtExecuteReq, text_row_res::TextRowRes,
      },
      status::Status,
    },
  },
  misc::{LeaseMut, Stream, Usize, Vector, partitioned_filled_buffer::PartitionedFilledBuffer},
};

impl<E, EB, S> MysqlExecutor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  #[inline]
  pub(crate) async fn fetch_cmd<const IS_SINGLE: bool>(
    net_buffer: &mut PartitionedFilledBuffer,
    sequence_id: &mut u8,
    stmt: Option<&MysqlStatement<'_>>,
    stream: &mut S,
    vb: &mut Vector<(bool, Range<usize>)>,
    mut cb_end: impl FnMut(u64) -> Result<(), E>,
    mut cb_rslt: impl FnMut(MysqlRecord<'_, E>) -> Result<(), E>,
  ) -> Result<(), E> {
    let smre = u16::from(Status::ServerMoreResultsExists);
    let mut has_at_least_one_record = false;
    loop {
      fetch_msg(net_buffer, sequence_id, stream).await?;
      let [first0, rest0 @ ..] = net_buffer._current() else {
        return Err(E::from(MysqlError::InvalidFetchBytes.into()));
      };
      let mut local_rest = rest0;
      if *first0 == 0 || *first0 == 255 {
        let res: OkRes = decode(&mut local_rest, ())?;
        if u16::from(res.status) & smre == smre {
          continue;
        }
        cb_end(res.affected_rows)?;
        return Ok(());
      }

      let columns_lenenc: Lenenc = decode(&mut local_rest, ())?;
      let columns = Usize::try_from(columns_lenenc.0)?.into_usize();

      for _ in 0..columns {
        let res: ColumnRes = fetch_protocol(net_buffer, sequence_id, stream).await?;
        let _column = Column::from_column_res(&res);
      }

      loop {
        fetch_msg(net_buffer, sequence_id, stream).await?;
        let mut current = net_buffer._current();
        if current.first() == Some(&254) && current.len() < 9 {
          let res: EofRes = decode(&mut current, ())?;
          cb_end(0)?;
          if u16::from(res.status) & smre == smre {
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
        if let Some(elem) = stmt {
          let begin = vb.len();
          let res: BinaryRowRes<'_> = decode(&mut current, (elem, &mut *vb))?;
          let rec = MysqlRecord {
            bytes: res.0,
            phantom: PhantomData,
            stmt: elem.clone(),
            values_bytes_offsets: vb.get(begin..).unwrap_or_default(),
          };
          cb_rslt(rec)?;
        } else {
          let _row: TextRowRes = decode(&mut current, (columns, &mut *vb))?;
        };
      }
    }
  }

  #[inline]
  pub(crate) async fn write_send_await_stmt<'stmts, RV, SC, const IS_SINGLE: bool>(
    (capabilities, sequence_id): (&mut u64, &mut u8),
    enc_buffer: &mut Vector<u8>,
    net_buffer: &mut PartitionedFilledBuffer,
    rv: RV,
    sc: SC,
    stmts: &'stmts mut MysqlStatements,
    stream: &mut S,
    vb: &mut Vector<(bool, Range<usize>)>,
    cb: impl FnMut(MysqlRecord<'_, E>) -> Result<(), E>,
  ) -> Result<MysqlStatement<'stmts>, E>
  where
    RV: RecordValues<Mysql<E>>,
    SC: StmtCmd,
  {
    let (_, _, stmt) = Self::write_send_await_stmt_prot(
      (capabilities, sequence_id),
      enc_buffer,
      net_buffer,
      sc,
      stmts,
      stream,
    )
    .await?;
    send_packet(
      (capabilities, sequence_id),
      enc_buffer,
      StmtExecuteReq { rv, stmt: &stmt },
      stream,
    )
    .await?;
    Self::fetch_cmd::<IS_SINGLE>(net_buffer, sequence_id, Some(&stmt), stream, vb, |_| Ok(()), cb)
      .await?;
    Ok(stmt)
  }
}
