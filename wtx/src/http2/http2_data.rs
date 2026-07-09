use crate::{
  http::{HttpRecvParams, U31},
  http2::{Http2Buffer, http_send_params::HttpSendParams, window::Windows},
  misc::{Lease, LeaseMut},
};

/// Internal resource used in every new instance of `Http2`.
#[derive(Debug)]
pub struct Http2Data<const IS_CLIENT: bool> {
  frame_reader_error: Option<crate::Error>,
  hb: Http2Buffer,
  hp: HttpRecvParams,
  hps: HttpSendParams,
  last_stream_id: U31,
  recv_streams_num: u32,
  windows: Windows,
}

impl<const IS_CLIENT: bool> Http2Data<IS_CLIENT> {
  pub(crate) fn new(hb: Http2Buffer, hp: HttpRecvParams) -> Self {
    let hps = HttpSendParams::default();
    let windows = Windows::initial(&hp, &hps);
    Self {
      frame_reader_error: None,
      hb,
      hp,
      hps,
      last_stream_id: if IS_CLIENT { U31::ONE } else { U31::ZERO },
      recv_streams_num: 0,
      windows,
    }
  }

  pub(crate) fn parts_mut(&mut self) -> Http2DataPartsMut<'_, IS_CLIENT> {
    Http2DataPartsMut {
      frame_reader_error: &mut self.frame_reader_error,
      hb: &mut self.hb,
      hp: &mut self.hp,
      hps: &mut self.hps,
      last_stream_id: &mut self.last_stream_id,
      recv_streams_num: &mut self.recv_streams_num,
      windows: &mut self.windows,
    }
  }
}

impl<const IS_CLIENT: bool> Lease<Http2Data<IS_CLIENT>> for Http2Data<IS_CLIENT> {
  #[inline]
  fn lease(&self) -> &Http2Data<IS_CLIENT> {
    self
  }
}

impl<const IS_CLIENT: bool> LeaseMut<Http2Data<IS_CLIENT>> for Http2Data<IS_CLIENT> {
  #[inline]
  fn lease_mut(&mut self) -> &mut Http2Data<IS_CLIENT> {
    self
  }
}

pub(crate) struct Http2DataPartsMut<'instance, const IS_CLIENT: bool> {
  pub(crate) frame_reader_error: &'instance mut Option<crate::Error>,
  pub(crate) hb: &'instance mut Http2Buffer,
  pub(crate) hp: &'instance mut HttpRecvParams,
  pub(crate) hps: &'instance mut HttpSendParams,
  pub(crate) last_stream_id: &'instance mut U31,
  pub(crate) recv_streams_num: &'instance mut u32,
  pub(crate) windows: &'instance mut Windows,
}
