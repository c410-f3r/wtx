use crate::{
  http2::{ErrorCode, FrameHeaderTy, FrameInit, StreamId},
  misc::{ByteVector, _unlikely_elem},
};

#[derive(Debug, Eq, PartialEq)]
pub struct GoAwayFrame<'data> {
  data: &'data [u8],
  error_code: ErrorCode,
  last_stream_id: StreamId,
}

impl<'data> GoAwayFrame<'data> {
  pub(crate) fn read(data: &'data [u8]) -> crate::Result<Self> {
    let [a, b, c, d, e, f, g, h, data @ ..] = data else {
      return Err(crate::http2::ErrorCode::FrameSizeError.into());
    };
    Ok(Self {
      data,
      error_code: u32::from_be_bytes([*e, *f, *g, *h]).try_into()?,
      last_stream_id: StreamId::from(u32::from_be_bytes([*a, *b, *c, *d])),
    })
  }

  pub(crate) fn write(&self, wb: &mut ByteVector) -> crate::Result<()> {
    let Ok(payload_len) = u32::try_from(self.data.len()) else {
      return _unlikely_elem(Err(crate::http2::ErrorCode::FrameSizeError.into()));
    };
    let len = payload_len.wrapping_add(8);
    wb.extend_from_slices(&[
      FrameInit::new(0, len, StreamId::ZERO, FrameHeaderTy::GoAway).bytes().as_slice(),
      self.last_stream_id.to_be_bytes().as_slice(),
      u32::from(self.error_code).to_be_bytes().as_slice(),
      self.data,
    ]);
    Ok(())
  }
}
