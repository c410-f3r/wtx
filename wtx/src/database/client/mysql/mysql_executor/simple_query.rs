use core::ops::Range;

use crate::{
  database::client::mysql::{
    ExecutorBuffer, MysqlExecutor, MysqlStatement, misc::send_packet,
    mysql_protocol::query_req::QueryReq,
  },
  misc::{LeaseMut, Stream, Vector, partitioned_filled_buffer::PartitionedFilledBuffer},
};

impl<E, EB, S> MysqlExecutor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  #[inline]
  pub(crate) async fn simple_query_execute(
    (capabilities, sequence_id): (&mut u64, &mut u8),
    cmd: &str,
    encode_buffer: &mut Vector<u8>,
    net_buffer: &mut PartitionedFilledBuffer,
    records_params: &mut Vector<(Range<usize>, Range<usize>)>,
    stream: &mut S,
    values_params: &mut Vector<(bool, Range<usize>)>,
    mut cb: impl FnMut(u64) -> Result<(), E>,
  ) -> Result<(), E> {
    send_packet(
      (capabilities, sequence_id),
      encode_buffer,
      QueryReq { query: cmd.as_bytes() },
      stream,
    )
    .await?;
    Self::fetch_cmd::<false, false>(
      *capabilities,
      net_buffer,
      records_params,
      sequence_id,
      &MysqlStatement::default(),
      stream,
      values_params,
      |num| {
        cb(num)?;
        Ok(())
      },
      |_| Ok(()),
    )
    .await?;
    Ok(())
  }
}
