use crate::{
  collection::{TryExtend, Vector},
  database::client::mysql::{
    ExecutorBuffer, MysqlExecutor, MysqlRecord, MysqlRecords, MysqlStatements, misc::send_packet,
    protocol::query_req::QueryReq,
  },
  misc::{LeaseMut, net::PartitionedFilledBuffer},
  stream::Stream,
};
use core::ops::Range;

impl<E, EB, S> MysqlExecutor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  pub(crate) async fn simple_query_execute<'exec, B>(
    buffer: &mut B,
    (capabilities, sequence_id): (&mut u64, &mut u8),
    cmd: &str,
    encode_buffer: &mut Vector<u8>,
    net_buffer: &'exec mut PartitionedFilledBuffer,
    records_params: &'exec mut Vector<(Range<usize>, Range<usize>)>,
    stmts: &'exec mut MysqlStatements,
    stream: &mut S,
    values_params: &'exec mut Vector<(bool, Range<usize>)>,
    cb: impl FnMut(MysqlRecord<'_, E>) -> Result<(), E>,
  ) -> Result<(), E>
  where
    B: TryExtend<[MysqlRecords<'exec, E>; 1]>,
  {
    send_packet(
      (capabilities, sequence_id),
      encode_buffer,
      QueryReq { query: cmd.as_bytes() },
      stream,
    )
    .await?;
    Self::fetch_text_cmd(
      buffer,
      *capabilities,
      net_buffer,
      records_params,
      sequence_id,
      stmts,
      stream,
      values_params,
      cb,
    )
    .await?;
    Ok(())
  }
}
