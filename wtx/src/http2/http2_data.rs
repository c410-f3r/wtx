use crate::{
  http2::{
    Http2Buffer, Http2Params, http2_params_send::Http2ParamsSend, u31::U31, window::Windows,
  },
  misc::{Lease, LeaseMut, StreamWriter},
};

/// Internal resource used in every new instance of `Http2`.
#[derive(Debug)]
pub struct Http2Data<HB, SW, const IS_CLIENT: bool> {
  frame_reader_error: Option<crate::Error>,
  hb: HB,
  hp: Http2Params,
  hps: Http2ParamsSend,
  last_stream_id: U31,
  recv_streams_num: u32,
  stream_writer: SW,
  windows: Windows,
}

impl<HB, SW, const IS_CLIENT: bool> Http2Data<HB, SW, IS_CLIENT>
where
  HB: LeaseMut<Http2Buffer>,
  SW: StreamWriter,
{
  #[inline]
  pub(crate) fn new(hb: HB, hp: Http2Params, stream_writer: SW) -> Self {
    let hps = Http2ParamsSend::default();
    let windows = Windows::initial(&hp, &hps);
    Self {
      frame_reader_error: None,
      hb,
      hp,
      hps,
      last_stream_id: if IS_CLIENT { U31::ONE } else { U31::ZERO },
      recv_streams_num: 0,
      stream_writer,
      windows,
    }
  }

  #[inline]
  pub(crate) fn parts_mut(&mut self) -> Http2DataPartsMut<'_, SW, IS_CLIENT> {
    Http2DataPartsMut {
      frame_reader_error: &mut self.frame_reader_error,
      hb: self.hb.lease_mut(),
      hp: &mut self.hp,
      hps: &mut self.hps,
      last_stream_id: &mut self.last_stream_id,
      recv_streams_num: &mut self.recv_streams_num,
      stream_writer: &mut self.stream_writer,
      windows: &mut self.windows,
    }
  }
}

impl<HB, SW, const IS_CLIENT: bool> Lease<Http2Data<HB, SW, IS_CLIENT>>
  for Http2Data<HB, SW, IS_CLIENT>
{
  #[inline]
  fn lease(&self) -> &Http2Data<HB, SW, IS_CLIENT> {
    self
  }
}

impl<HB, SW, const IS_CLIENT: bool> LeaseMut<Http2Data<HB, SW, IS_CLIENT>>
  for Http2Data<HB, SW, IS_CLIENT>
{
  #[inline]
  fn lease_mut(&mut self) -> &mut Http2Data<HB, SW, IS_CLIENT> {
    self
  }
}

pub(crate) struct Http2DataPartsMut<'instance, SW, const IS_CLIENT: bool> {
  pub(crate) frame_reader_error: &'instance mut Option<crate::Error>,
  pub(crate) hb: &'instance mut Http2Buffer,
  pub(crate) hp: &'instance mut Http2Params,
  pub(crate) hps: &'instance mut Http2ParamsSend,
  pub(crate) last_stream_id: &'instance mut U31,
  pub(crate) recv_streams_num: &'instance mut u32,
  pub(crate) stream_writer: &'instance mut SW,
  pub(crate) windows: &'instance mut Windows,
}
