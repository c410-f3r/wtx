use crate::{
  database::{
    client::postgres::{
      executor::commons::FetchWithStmtCommons,
      message::{Message, MessageTy},
      statements::Statement,
      Executor, ExecutorBuffer, Postgres, PostgresError, Record,
    },
    RecordValues,
  },
  misc::{ConnectionState, LeaseMut, PartitionedFilledBuffer, Stream, Vector, _read_until},
};
use core::ops::Range;

impl<E, EB, S> Executor<E, EB, S>
where
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
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
        MessageTy::DataRow(len) => {
          data_row_msg_range = Some((len, nb._current_range()));
        }
        MessageTy::ReadyForQuery => break,
        MessageTy::CommandComplete(_) | MessageTy::EmptyQueryResponse => {}
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

  pub(crate) async fn fetch_msg_from_stream<'nb>(
    cs: &mut ConnectionState,
    nb: &'nb mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> crate::Result<Message<'nb>> {
    let tag = Self::fetch_representative_msg_from_stream(nb, stream).await?;
    Ok(Message { tag, ty: MessageTy::try_from((cs, nb._current()))? })
  }

  async fn fetch_one_header_from_stream(
    nb: &mut PartitionedFilledBuffer,
    read: &mut usize,
    stream: &mut S,
  ) -> crate::Result<(u8, usize)> {
    let buffer = nb._following_rest_mut();
    let [mt_n, b, c, d, e] = _read_until::<5, S>(buffer, read, 0, stream).await?;
    let len: usize = u32::from_be_bytes([b, c, d, e]).try_into()?;
    Ok((mt_n, len.wrapping_add(1)))
  }

  async fn fetch_one_msg_from_stream<'nb>(
    nb: &'nb mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> crate::Result<u8> {
    let mut read = nb._following_len();
    let (ty, len) = Self::fetch_one_header_from_stream(nb, &mut read, stream).await?;
    Self::fetch_one_payload_from_stream(len, nb, &mut read, stream).await?;
    let current_end_idx = nb._current_end_idx();
    nb._set_indices(current_end_idx, len, read.wrapping_sub(len))?;
    Ok(ty)
  }

  async fn fetch_one_payload_from_stream(
    len: usize,
    nb: &mut PartitionedFilledBuffer,
    read: &mut usize,
    stream: &mut S,
  ) -> crate::Result<()> {
    let mut is_payload_filled = false;
    nb._reserve(len)?;
    for _ in 0..=len {
      if *read >= len {
        is_payload_filled = true;
        break;
      }
      *read = read.wrapping_add(
        stream.read(nb._following_rest_mut().get_mut(*read..).unwrap_or_default()).await?,
      );
    }
    if !is_payload_filled {
      return Err(crate::Error::UnexpectedBufferState);
    }
    Ok(())
  }

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
