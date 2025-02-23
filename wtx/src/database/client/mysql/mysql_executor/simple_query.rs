use core::ops::Range;

use crate::{
  database::client::mysql::{
    ExecutorBuffer, MysqlExecutor, misc::send_packet, mysql_protocol::query_req::QueryReq,
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
    enc_buffer: &mut Vector<u8>,
    net_buffer: &mut PartitionedFilledBuffer,
    stream: &mut S,
    vb: &mut Vector<(bool, Range<usize>)>,
    mut cb: impl FnMut(u64) -> Result<(), E>,
  ) -> Result<(), E> {
    send_packet(
      (capabilities, sequence_id),
      enc_buffer,
      QueryReq { query: cmd.as_bytes() },
      stream,
    )
    .await?;
    Self::fetch_cmd::<false>(
      net_buffer,
      sequence_id,
      None,
      stream,
      vb,
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
