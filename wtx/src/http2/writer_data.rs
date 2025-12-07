use crate::collection::Vector;

#[derive(Debug)]
pub(crate) struct WriterData<SW> {
  pub(crate) hpack_enc_buffer: Vector<u8>,
  pub(crate) stream_writer: SW,
}

impl<SW> WriterData<SW> {
  pub(crate) fn new(hpack_enc_buffer: Vector<u8>, stream_writer: SW) -> Self {
    Self { hpack_enc_buffer, stream_writer }
  }
}
