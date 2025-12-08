#[derive(Debug)]
pub(crate) struct WriterData<SW> {
  pub(crate) stream_writer: SW,
}

impl<SW> WriterData<SW> {
  pub(crate) fn new(stream_writer: SW) -> Self {
    Self { stream_writer }
  }
}
