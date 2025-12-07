use crate::misc::net::PartitionedFilledBuffer;

#[derive(Debug)]
pub(crate) struct ReaderData<SR> {
  pub(crate) pfb: PartitionedFilledBuffer,
  pub(crate) stream_reader: SR,
}

impl<SR> ReaderData<SR> {
  pub(crate) fn new(pfb: PartitionedFilledBuffer, stream_reader: SR) -> Self {
    Self { pfb, stream_reader }
  }
}
