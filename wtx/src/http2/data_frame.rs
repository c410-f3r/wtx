use crate::{
  http2::{misc::trim_frame_pad, FrameHeaderTy, FrameInit, StreamId},
  misc::{Vector, _unlikely_elem},
};

const ALL: u8 = END_STREAM | PADDED;
const END_STREAM: u8 = 0b0000_0001;
const PADDED: u8 = 0b0000_1000;

#[derive(Debug, Eq, PartialEq)]
pub struct DataFrame<'data> {
  pub(crate) data: &'data [u8],
  pub(crate) data_len: u32,
  pub(crate) flags: u8,
  pub(crate) pad_len: Option<u8>,
  pub(crate) stream_id: StreamId,
}

impl<'data> DataFrame<'data> {
  pub fn new(data: &'data [u8], data_len: u32, stream_id: StreamId) -> Self {
    Self { data, data_len, flags: 0, pad_len: None, stream_id }
  }

  pub(crate) fn bytes(&self) -> [u8; 9] {
    FrameInit::new(self.flags.into(), 0, self.stream_id, FrameHeaderTy::Data).bytes()
  }

  pub(crate) fn read(mut data: &'data [u8], fi: FrameInit) -> crate::Result<Self> {
    if fi.stream_id.is_zero() {
      return _unlikely_elem(Err(crate::http2::ErrorCode::ProtocolError.into()));
    }
    let flags = fi.flag & ALL;
    let pad_len = trim_frame_pad(&mut data, flags)?;
    let Ok(data_len) = u32::try_from(data.len()) else {
      return _unlikely_elem(Err(crate::http2::ErrorCode::ProtocolError.into()));
    };
    Ok(Self { data, data_len, flags, pad_len, stream_id: fi.stream_id })
  }
}
