use crate::http2::{
  misc::{protocol_err, trim_frame_pad},
  CommonFlags, FrameInit, FrameInitTy, Http2Error, U31,
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct DataFrame {
  cf: CommonFlags,
  data_len: U31,
  pad_len: Option<u8>,
  stream_id: U31,
}

impl DataFrame {
  #[inline]
  pub(crate) const fn new(data_len: U31, stream_id: U31) -> Self {
    Self { cf: CommonFlags::empty(), data_len, pad_len: None, stream_id }
  }

  #[inline]
  pub(crate) const fn bytes(&self) -> [u8; 9] {
    FrameInit::new(self.cf, self.data_len.u32(), self.stream_id, FrameInitTy::Data).bytes()
  }

  #[inline]
  pub(crate) const fn data_len(&self) -> U31 {
    self.data_len
  }

  #[inline]
  pub(crate) const fn has_eos(&self) -> bool {
    self.cf.has_eos()
  }

  #[inline]
  pub(crate) fn read(mut data: &[u8], mut fi: FrameInit) -> crate::Result<(Self, &[u8])> {
    if fi.stream_id.is_zero() {
      return Err(protocol_err(Http2Error::InvalidDataFrameZeroId));
    }
    fi.cf.only_eos_pad();
    let pad_len = trim_frame_pad(fi.cf, &mut data)?;
    let Ok(data_len) = u32::try_from(data.len()).map(U31::from_u32) else {
      return Err(protocol_err(Http2Error::InvalidDataFrameDataLen));
    };
    Ok((Self { cf: fi.cf, data_len, pad_len, stream_id: fi.stream_id }, data))
  }

  #[inline]
  pub(crate) fn set_eos(&mut self) {
    self.cf.set_eos();
  }
}
