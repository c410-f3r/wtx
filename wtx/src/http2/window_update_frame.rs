use crate::{
  http2::{FrameHeaderTy, FrameInit, StreamId},
  misc::ByteVector,
};

#[derive(Debug, Eq, PartialEq)]
pub struct WindowUpdateFrame {
  size_increment: u32,
  stream_id: StreamId,
}

impl WindowUpdateFrame {
  pub(crate) fn read(bytes: &[u8], fi: FrameInit) -> crate::Result<Self> {
    let [a, b, c, d] = bytes else {
      return Err(crate::http2::ErrorCode::FrameSizeError.into());
    };
    let size_increment = u32::from_be_bytes([*a, *b, *c, *d]);
    if size_increment == 0 {
      return Err(crate::http2::ErrorCode::ProtocolError.into());
    }
    Ok(Self { size_increment, stream_id: fi.stream_id })
  }

  pub(crate) fn write(&self, wb: &mut ByteVector) {
    wb.extend_from_slices(&[
      FrameInit::new(0, 4, self.stream_id, FrameHeaderTy::WindowUpdate).bytes().as_slice(),
      self.size_increment.to_be_bytes().as_slice(),
    ]);
  }
}
