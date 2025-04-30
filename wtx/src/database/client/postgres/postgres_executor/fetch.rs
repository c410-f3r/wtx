use crate::{
  collection::Vector,
  database::{
    DatabaseError, RecordValues,
    client::postgres::{
      ExecutorBuffer, Postgres, PostgresError, PostgresExecutor, PostgresRecord, PostgresStatement,
      message::{Message, MessageTy},
      postgres_executor::commons::FetchWithStmtCommons,
    },
  },
  misc::{
    ConnectionState, LeaseMut, Usize,
    net::{PartitionedFilledBuffer, read_header, read_payload},
  },
  stream::Stream,
};
use core::ops::Range;

impl<E, EB, S> PostgresExecutor<E, EB, S>
where
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  #[inline]
  pub(crate) async fn write_send_await_fetch_with_stmt_wo_prot<'any, RV>(
    fwsc: &mut FetchWithStmtCommons<'_, S>,
    net_buffer: &'any mut PartitionedFilledBuffer,
    rv: RV,
    stmt: PostgresStatement<'any>,
    stmt_cmd_id_array: &[u8],
    values_params: &'any mut Vector<(bool, Range<usize>)>,
  ) -> Result<PostgresRecord<'any, E>, E>
  where
    E: From<crate::Error>,
    RV: RecordValues<Postgres<E>>,
  {
    Self::write_send_await_stmt_initial(fwsc, net_buffer, rv, &stmt, stmt_cmd_id_array).await?;
    let mut data_row_msg_range = None;
    loop {
      let msg = Self::fetch_msg_from_stream(fwsc.cs, net_buffer, fwsc.stream).await?;
      match msg.ty {
        MessageTy::CommandComplete(_) | MessageTy::EmptyQueryResponse => {}
        MessageTy::DataRow(len) => {
          data_row_msg_range = Some((len, net_buffer._current_range()));
        }
        MessageTy::ReadyForQuery => break,
        _ => {
          return Err(E::from(
            PostgresError::UnexpectedDatabaseMessage { received: msg.tag }.into(),
          ));
        }
      }
    }
    if let Some((record_bytes, values_len)) = data_row_msg_range.and_then(|(len, range)| {
      let record_range = range.start.wrapping_add(7)..range.end;
      Some((net_buffer._all().get(record_range)?, len))
    }) {
      Ok(PostgresRecord::parse(record_bytes, stmt, values_len, values_params)?)
    } else {
      Err(E::from(DatabaseError::MissingRecord.into()))
    }
  }

  #[inline]
  pub(crate) async fn fetch_msg_from_stream<'nb>(
    cs: &mut ConnectionState,
    net_buffer: &'nb mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> crate::Result<Message<'nb>> {
    let tag = Self::fetch_representative_msg_from_stream(net_buffer, stream).await?;
    Ok(Message { tag, ty: MessageTy::try_from((cs, net_buffer._current()))? })
  }

  // | Ty | Len | Payload |
  // | 1  |  4  |    x    |
  //
  // The value of `Len` is payload length plus 4, therefore, the frame length is `Len` plus 1.
  #[inline]
  async fn fetch_one_msg_from_stream(
    net_buffer: &mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> crate::Result<u8> {
    net_buffer._reserve(5)?;
    let mut read = net_buffer._following_len();
    let buffer = net_buffer._following_rest_mut();
    let [a, b, c, d, e] = read_header::<0, 5, S>(buffer, &mut read, stream).await?;
    let len = Usize::from(u32::from_be_bytes([b, c, d, e])).into_usize().wrapping_add(1);
    read_payload((0, len), net_buffer, &mut read, stream).await?;
    Ok(a)
  }

  #[inline]
  async fn fetch_representative_msg_from_stream(
    net_buffer: &mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> crate::Result<u8> {
    let mut tag = Self::fetch_one_msg_from_stream(&mut *net_buffer, stream).await?;
    while tag == b'N' {
      tag = Self::fetch_one_msg_from_stream(net_buffer, stream).await?;
    }
    Ok(tag)
  }
}
