use crate::{
  http::ReqResBuffer,
  http2::{http2_params_send::Http2ParamsSend, Http2Buffer, Http2Params, Windows, U31},
  misc::{Lease, LeaseMut, StreamWriter},
};
use core::marker::PhantomData;

/// Internal resource used in every new instance of `Http2`.
#[derive(Debug)]
pub struct Http2Data<HB, RRB, SW, const IS_CLIENT: bool> {
  hb: HB,
  hp: Http2Params,
  hps: Http2ParamsSend,
  last_stream_id: U31,
  phantom: PhantomData<RRB>,
  recv_streams_num: u32,
  stream_writer: SW,
  windows: Windows,
}

impl<HB, RRB, SW, const IS_CLIENT: bool> Http2Data<HB, RRB, SW, IS_CLIENT>
where
  HB: LeaseMut<Http2Buffer<RRB>>,
  RRB: LeaseMut<ReqResBuffer>,
  SW: StreamWriter,
{
  #[inline]
  pub(crate) fn new(hb: HB, hp: Http2Params, stream_writer: SW) -> Self {
    let hps = Http2ParamsSend::default();
    let windows = Windows::initial(&hp, &hps);
    Self {
      hb,
      hp,
      hps,
      last_stream_id: if IS_CLIENT { U31::ONE } else { U31::ZERO },
      phantom: PhantomData,
      recv_streams_num: 0,
      stream_writer,
      windows,
    }
  }

  #[inline]
  pub(crate) fn parts_mut(&mut self) -> Http2DataPartsMut<'_, RRB, SW> {
    Http2DataPartsMut {
      hb: self.hb.lease_mut(),
      last_stream_id: &mut self.last_stream_id,
      hp: &mut self.hp,
      hps: &mut self.hps,
      recv_streams_num: &mut self.recv_streams_num,
      stream_writer: &mut self.stream_writer,
      windows: &mut self.windows,
    }
  }
}

impl<HB, RRB, SW, const IS_CLIENT: bool> Lease<Http2Data<HB, RRB, SW, IS_CLIENT>>
  for Http2Data<HB, RRB, SW, IS_CLIENT>
{
  #[inline]
  fn lease(&self) -> &Http2Data<HB, RRB, SW, IS_CLIENT> {
    self
  }
}

impl<HB, RRB, SW, const IS_CLIENT: bool> LeaseMut<Http2Data<HB, RRB, SW, IS_CLIENT>>
  for Http2Data<HB, RRB, SW, IS_CLIENT>
{
  #[inline]
  fn lease_mut(&mut self) -> &mut Http2Data<HB, RRB, SW, IS_CLIENT> {
    self
  }
}

pub(crate) struct Http2DataPartsMut<'instance, RRB, SW> {
  pub(crate) hb: &'instance mut Http2Buffer<RRB>,
  pub(crate) hp: &'instance mut Http2Params,
  pub(crate) hps: &'instance mut Http2ParamsSend,
  pub(crate) last_stream_id: &'instance mut U31,
  pub(crate) recv_streams_num: &'instance mut u32,
  pub(crate) stream_writer: &'instance mut SW,
  pub(crate) windows: &'instance mut Windows,
}
