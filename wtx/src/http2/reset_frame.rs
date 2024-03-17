use crate::{
  http2::{ErrorCode, FrameHeaderTy, FrameInit, StreamId},
  misc::ByteVector,
};

#[derive(Debug, Eq, PartialEq)]
pub struct ResetFrame {
  error_code: ErrorCode,
  stream_id: StreamId,
}

impl ResetFrame {
  pub(crate) fn read(bytes: &[u8], fi: FrameInit) -> crate::Result<Self> {
    let [a, b, c, d] = bytes else {
      return Err(crate::http2::ErrorCode::FrameSizeError.into());
    };
    Ok(Self {
      error_code: u32::from_be_bytes([*a, *b, *c, *d]).try_into()?,
      stream_id: fi.stream_id,
    })
  }

  pub(crate) fn write(&self, wb: &mut ByteVector) {
    let bytes = FrameInit::new(0, 4, self.stream_id, FrameHeaderTy::Reset).bytes();
    wb.extend_from_slices(&[bytes.as_slice(), u32::from(self.error_code).to_be_bytes().as_slice()]);
  }
}
