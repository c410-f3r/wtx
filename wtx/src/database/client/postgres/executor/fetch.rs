use crate::{
  database::{
    client::postgres::{
      executor::commons::FetchWithStmtCommons,
      message::{Message, MessageTy},
      statements::statement::Statement,
      Executor, ExecutorBuffer, Postgres, PostgresError, Record,
    },
    RecordValues,
  },
  misc::{
    partitioned_filled_buffer::PartitionedFilledBuffer, ConnectionState, LeaseMut, Stream, Usize,
    Vector, _read_header, _read_payload,
  },
};
use core::ops::Range;

impl<E, EB, S> Executor<E, EB, S>
where
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  #[inline]
  pub(crate) async fn write_send_await_fetch_with_stmt_wo_prot<'any, RV>(
    fwsc: &mut FetchWithStmtCommons<'_, S>,
    nb: &'any mut PartitionedFilledBuffer,
    rv: RV,
    stmt: Statement<'any>,
    stmt_id_str: &str,
    vb: &'any mut Vector<(bool, Range<usize>)>,
  ) -> Result<Record<'any, E>, E>
  where
    E: From<crate::Error>,
    RV: RecordValues<Postgres<E>>,
  {
    Self::write_send_await_stmt_initial(fwsc, nb, rv, &stmt, stmt_id_str).await?;
    let mut data_row_msg_range = None;
    loop {
      let msg = Self::fetch_msg_from_stream(fwsc.cs, nb, fwsc.stream).await?;
      match msg.ty {
        MessageTy::CommandComplete(_) | MessageTy::EmptyQueryResponse => {}
        MessageTy::DataRow(len) => {
          data_row_msg_range = Some((len, nb._current_range()));
        }
        MessageTy::ReadyForQuery => break,
        _ => {
          return Err(E::from(
            PostgresError::UnexpectedDatabaseMessage { received: msg.tag }.into(),
          ))
        }
      }
    }
    if let Some((record_bytes, len)) = data_row_msg_range.and_then(|(len, range)| {
      let record_range = range.start.wrapping_add(7)..range.end;
      Some((nb._buffer().get(record_range)?, len))
    }) {
      Record::parse(record_bytes, 0..record_bytes.len(), stmt, vb, len).map_err(From::from)
    } else {
      Err(E::from(PostgresError::NoRecord.into()))
    }
  }

  #[inline]
  pub(crate) async fn fetch_msg_from_stream<'nb>(
    cs: &mut ConnectionState,
    nb: &'nb mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> crate::Result<Message<'nb>> {
    let tag = Self::fetch_representative_msg_from_stream(nb, stream).await?;
    Ok(Message { tag, ty: MessageTy::try_from((cs, nb._current()))? })
  }

  // | Ty | Len | Payload |
  // | 1  |  4  |    x    |
  //
  // The value of `Len` is payload length plus 4, therefore, the frame length is `Len` plus 1.
  #[inline]
  async fn fetch_one_msg_from_stream<'nb>(
    nb: &'nb mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> crate::Result<u8> {
    nb._reserve(5)?;
    let mut read = nb._following_len();
    let buffer = nb._following_rest_mut();
    let [a, b, c, d, e] = _read_header::<0, 5, S>(buffer, &mut read, stream).await?;
    let len = Usize::from(u32::from_be_bytes([b, c, d, e])).into_usize().wrapping_add(1);
    _read_payload((0, len), nb, &mut read, stream).await?;
    Ok(a)
  }

  #[inline]
  async fn fetch_representative_msg_from_stream<'nb>(
    nb: &'nb mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> crate::Result<u8> {
    let mut tag = Self::fetch_one_msg_from_stream(&mut *nb, stream).await?;
    while tag == b'N' {
      tag = Self::fetch_one_msg_from_stream(nb, stream).await?;
    }
    Ok(tag)
  }
}
