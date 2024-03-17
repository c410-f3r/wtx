use crate::{
  http2::{FrameHeaderTy, FrameInit, StreamId, ACK_MASK},
  misc::ByteVector,
};

#[derive(Debug, Eq, PartialEq)]
pub struct PingFrame {
  ack: bool,
  payload: [u8; 8],
}

impl PingFrame {
  pub(crate) fn read(bytes: &[u8], fi: FrameInit) -> crate::Result<Self> {
    if fi.stream_id.is_not_zero() {
      return Err(crate::http2::ErrorCode::ProtocolError.into());
    }
    let [a, b, c, d, e, f, g, h] = bytes else {
      return Err(crate::http2::ErrorCode::FrameSizeError.into());
    };
    Ok(Self { ack: fi.flag & ACK_MASK != 0, payload: [*a, *b, *c, *d, *e, *f, *g, *h] })
  }

  pub(crate) fn write(&self, wb: &mut ByteVector) {
    let flag = if self.ack { ACK_MASK } else { 0 };
    wb.extend_from_slices(&[
      FrameInit::new(flag, 8, StreamId::ZERO, FrameHeaderTy::Ping).bytes().as_slice(),
      self.payload.as_slice(),
    ]);
  }
}
