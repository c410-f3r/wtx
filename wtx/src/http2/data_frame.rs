use crate::{
  http2::{misc::trim_frame_pad, FrameHeaderTy, FrameInit, EOS_MASK, PAD_MASK, U31},
  misc::_unlikely_elem,
};

const FLAG_MASK: u8 = EOS_MASK | PAD_MASK;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct DataFrame {
  data_len: U31,
  flag: u8,
  pad_len: Option<u8>,
  stream_id: U31,
}

impl DataFrame {
  pub(crate) fn new(data_len: U31, stream_id: U31) -> Self {
    Self { data_len, flag: 0, pad_len: None, stream_id }
  }

  pub(crate) fn bytes(&self) -> [u8; 9] {
    FrameInit::new(self.data_len.u32(), self.flag, self.stream_id, FrameHeaderTy::Data).bytes()
  }

  pub(crate) fn data_len(&self) -> U31 {
    self.data_len
  }

  pub(crate) fn is_eos(&self) -> bool {
    self.flag & EOS_MASK == EOS_MASK
  }

  pub(crate) fn read(mut data: &[u8], fi: FrameInit) -> crate::Result<Self> {
    if fi.stream_id.is_zero() {
      return _unlikely_elem(Err(crate::http2::ErrorCode::ProtocolError.into()));
    }
    let flag = fi.flags & FLAG_MASK;
    let pad_len = trim_frame_pad(&mut data, flag)?;
    let Ok(data_len) = u32::try_from(data.len()).map(U31::from_u32) else {
      return _unlikely_elem(Err(crate::http2::ErrorCode::ProtocolError.into()));
    };
    Ok(Self { data_len, flag, pad_len, stream_id: fi.stream_id })
  }

  pub(crate) fn set_eos(&mut self) {
    self.flag |= EOS_MASK
  }
}
