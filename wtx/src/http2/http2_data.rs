use crate::{
  http::ReqResBuffer,
  http2::{
    http2_params_send::Http2ParamsSend,
    misc::{protocol_err, read_frame_until},
    FrameInitTy, Http2Buffer, Http2Error, Http2Params, ProcessReceiptFrameTy, Windows, U31,
  },
  misc::{Lease, LeaseMut, Stream},
};
use core::marker::PhantomData;

/// Internal resource used in every new instance of `Http2`.
#[derive(Debug)]
pub struct Http2Data<HB, RRB, S, const IS_CLIENT: bool> {
  hb: HB,
  hp: Http2Params,
  hps: Http2ParamsSend,
  is_conn_open: bool,
  last_stream_id: U31,
  phantom: PhantomData<RRB>,
  recv_streams_num: u32,
  stream: S,
  windows: Windows,
}

impl<HB, RRB, S, const IS_CLIENT: bool> Http2Data<HB, RRB, S, IS_CLIENT>
where
  HB: LeaseMut<Http2Buffer<RRB>>,
  RRB: LeaseMut<ReqResBuffer>,
  S: Stream,
{
  #[inline]
  pub(crate) fn new(hb: HB, hp: Http2Params, stream: S) -> Self {
    let hps = Http2ParamsSend::default();
    let windows = Windows::initial(&hp, &hps);
    Self {
      hb,
      hp,
      hps,
      is_conn_open: true,
      last_stream_id: if IS_CLIENT { U31::ONE } else { U31::ZERO },
      phantom: PhantomData,
      recv_streams_num: 0,
      stream,
      windows,
    }
  }

  #[inline]
  pub(crate) fn is_conn_open(&self) -> bool {
    self.is_conn_open
  }

  #[inline]
  pub(crate) fn parts_mut(&mut self) -> Http2DataPartsMut<'_, RRB, S> {
    Http2DataPartsMut {
      hb: self.hb.lease_mut(),
      last_stream_id: &mut self.last_stream_id,
      hp: &mut self.hp,
      hps: &mut self.hps,
      is_conn_open: &mut self.is_conn_open,
      stream: &mut self.stream,
      windows: &mut self.windows,
    }
  }

  /// Fetches and evaluates one or more arbitrary frames.
  #[inline]
  pub(crate) async fn process_receipt(&mut self) -> crate::Result<()> {
    let Http2Buffer {
      hpack_dec,
      hpack_enc,
      initial_server_header,
      pfb,
      scrp,
      sorp,
      uri_buffer,
      ..
    } = self.hb.lease_mut();
    if initial_server_header.is_some() {
      return Ok(());
    }
    let Some(fi) = read_frame_until(
      &mut self.windows,
      &mut self.hp,
      hpack_enc,
      &mut self.hps,
      &mut self.is_conn_open,
      pfb,
      scrp,
      sorp,
      &mut self.stream,
    )
    .await?
    else {
      return Ok(());
    };
    let prft = ProcessReceiptFrameTy {
      conn_windows: &mut self.windows,
      fi,
      hp: &mut self.hp,
      hpack_dec,
      is_conn_open: &mut self.is_conn_open,
      last_stream_id: &mut self.last_stream_id,
      pfb,
      phantom: PhantomData,
      recv_streams_num: &mut self.recv_streams_num,
      stream: &mut self.stream,
      uri_buffer,
    };
    match fi.ty {
      FrameInitTy::Data => {
        prft.data(sorp).await?;
      }
      FrameInitTy::Headers => {
        if scrp.contains_key(&prft.fi.stream_id) {
          return Err(protocol_err(Http2Error::UnexpectedNonControlFrame));
        }
        if IS_CLIENT {
          prft.header_client(sorp).await?;
        } else {
          prft.header_server(initial_server_header, sorp).await?;
        }
      }
      FrameInitTy::Reset => {
        prft.reset(scrp, sorp)?;
        return Ok(());
      }
      FrameInitTy::WindowUpdate if fi.stream_id.is_not_zero() => {
        prft.window_update(scrp, sorp)?;
      }
      _ => {
        return Err(protocol_err(Http2Error::UnexpectedConnFrame));
      }
    }
    Ok(())
  }
}

impl<HB, RRB, S, const IS_CLIENT: bool> Lease<Http2Data<HB, RRB, S, IS_CLIENT>>
  for Http2Data<HB, RRB, S, IS_CLIENT>
{
  #[inline]
  fn lease(&self) -> &Http2Data<HB, RRB, S, IS_CLIENT> {
    self
  }
}

impl<HB, RRB, S, const IS_CLIENT: bool> LeaseMut<Http2Data<HB, RRB, S, IS_CLIENT>>
  for Http2Data<HB, RRB, S, IS_CLIENT>
{
  #[inline]
  fn lease_mut(&mut self) -> &mut Http2Data<HB, RRB, S, IS_CLIENT> {
    self
  }
}

pub(crate) struct Http2DataPartsMut<'instance, RRB, S> {
  pub(crate) hb: &'instance mut Http2Buffer<RRB>,
  pub(crate) hp: &'instance mut Http2Params,
  pub(crate) hps: &'instance mut Http2ParamsSend,
  pub(crate) is_conn_open: &'instance mut bool,
  pub(crate) last_stream_id: &'instance mut U31,
  pub(crate) stream: &'instance mut S,
  pub(crate) windows: &'instance mut Windows,
}
