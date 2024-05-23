use crate::{
  http::{Method, StatusCode},
  http2::{
    http2_params_send::Http2ParamsSend,
    misc::{read_frame, send_go_away},
    window::WindowsPair,
    ContinuationFrame, DataFrame, FrameInit, FrameInitTy, HeadersFrame, HpackDecoder, Http2Error,
    Http2ErrorCode, Http2Params, ReqResBuffer, ResetStreamFrame, Sorp, StreamBuffer,
    StreamOverallRecvParams, StreamState, UriBuffer, WindowUpdateFrame, Windows, U31,
  },
  misc::{LeaseMut, PartitionedFilledBuffer, Queue, Stream},
};
use alloc::vec::Vec;
use core::marker::PhantomData;

#[derive(Debug)]
pub(crate) struct ProcessReceiptFrameTy<'instance, S, SB, const IS_CLIENT: bool> {
  pub(crate) conn_windows: &'instance mut Windows,
  pub(crate) fi: FrameInit,
  pub(crate) hp: &'instance mut Http2Params,
  pub(crate) hpack_dec: &'instance mut HpackDecoder,
  pub(crate) hps: &'instance Http2ParamsSend,
  pub(crate) is_conn_open: &'instance mut bool,
  pub(crate) last_stream_id: U31,
  pub(crate) pfb: &'instance mut PartitionedFilledBuffer,
  pub(crate) phantom: PhantomData<SB>,
  pub(crate) stream: &'instance mut S,
  pub(crate) uri_buffer: &'instance mut UriBuffer,
}

impl<'instance, S, SB, const IS_CLIENT: bool> ProcessReceiptFrameTy<'instance, S, SB, IS_CLIENT>
where
  S: Stream,
  SB: LeaseMut<StreamBuffer>,
{
  #[inline]
  pub(crate) async fn data(mut self, sorp: &mut Sorp<SB>) -> crate::Result<()> {
    let Some(socrp) = sorp.get_mut(&self.fi.stream_id) else {
      return Err(crate::Error::http2_go_away_generic(Http2Error::UnknownStreamReceiver));
    };
    self.manage_invalid_stream_state(socrp).await?;
    let Some(local_body_len) = socrp
      .body_len
      .checked_add(self.fi.data_len)
      .filter(|element| *element <= self.hp.max_body_len())
    else {
      return Err(crate::Error::http2_go_away_generic(Http2Error::LargeDataFrameLen));
    };
    socrp.body_len = local_body_len;
    let df = DataFrame::read(self.pfb._current(), self.fi)?;
    socrp.sb.lease_mut().rrb.body.reserve(self.pfb._current().len());
    socrp.sb.lease_mut().rrb.body.extend_from_slice(self.pfb._current())?;
    WindowsPair::new(self.conn_windows, &mut socrp.windows)
      .manage_recv(*self.is_conn_open, self.stream, self.fi.stream_id, df.data_len())
      .await?;
    if df.is_eos() {
      socrp.stream_state = StreamState::HalfClosedRemote;
    }
    Ok(())
  }

  #[inline]
  pub(crate) async fn header(
    self,
    initial_server_buffers: &mut Vec<SB>,
    initial_server_streams: &mut Queue<(Method, U31)>,
    sorp: &mut Sorp<SB>,
  ) -> crate::Result<()> {
    if IS_CLIENT {
      self.header_client(sorp).await
    } else {
      self.header_server(initial_server_buffers, initial_server_streams, sorp).await
    }
  }

  #[inline]
  pub(crate) async fn reset(self, sorp: &mut Sorp<SB>) -> crate::Result<()> {
    let Some(elem) = sorp.get_mut(&self.fi.stream_id) else {
      return Err(crate::Error::http2_go_away_generic(Http2Error::UnknownStreamReceiver));
    };
    let rsf = ResetStreamFrame::read(self.pfb._current(), self.fi)?;
    elem.stream_state = StreamState::Closed;
    Err(crate::Error::Http2ErrorReset(rsf.error_code(), None))
  }

  #[inline]
  pub(crate) async fn window_update(self, sorp: &mut Sorp<SB>) -> crate::Result<()> {
    let Some(elem) = sorp.get_mut(&self.fi.stream_id) else {
      return Err(crate::Error::http2_go_away_generic(Http2Error::UnknownStreamReceiver));
    };
    let wuf = WindowUpdateFrame::read(self.pfb._current(), self.fi)?;
    elem.windows.send.deposit(wuf.size_increment().i32());
    Ok(())
  }

  #[inline]
  async fn header_client(mut self, sorp: &mut Sorp<SB>) -> crate::Result<()> {
    let Some(elem) = sorp.get_mut(&self.fi.stream_id) else {
      return Err(crate::Error::http2_go_away_generic(Http2Error::UnknownStreamReceiver));
    };
    self.manage_invalid_stream_state(elem).await?;
    let is_eos = if elem.has_initial_header {
      self
        .read_header_and_continuations::<_, true>(&mut elem.sb.lease_mut().rrb, |_| Ok(()))
        .await?
        .0
    } else {
      let (is_eos, status_code) = self
        .read_header_and_continuations::<_, false>(&mut elem.sb.lease_mut().rrb, |hf| {
          hf.hsresh().status_code.ok_or(crate::Error::HTTP_MissingResponseStatusCode)
        })
        .await?;
      elem.has_initial_header = true;
      elem.status_code = status_code;
      is_eos
    };
    if is_eos {
      elem.stream_state = StreamState::Closed;
    }
    Ok(())
  }

  #[inline]
  async fn header_server(
    mut self,
    initial_server_buffers: &mut Vec<SB>,
    initial_server_streams: &mut Queue<(Method, U31)>,
    sorp: &mut Sorp<SB>,
  ) -> crate::Result<()> {
    #[inline]
    fn stream_state(is_eos: bool) -> StreamState {
      if is_eos {
        StreamState::HalfClosedRemote
      } else {
        StreamState::Open
      }
    }
    if let Some(elem) = sorp.get_mut(&self.fi.stream_id) {
      self.manage_invalid_stream_state(elem).await?;
      let (is_eos, _) = self
        .read_header_and_continuations::<_, true>(&mut elem.sb.lease_mut().rrb, |_| Ok(()))
        .await?;
      elem.stream_state = stream_state(is_eos);
    } else {
      if self.fi.stream_id.u32() % 2 == 0 {
        return Err(crate::Error::http2_go_away_generic(Http2Error::EvenStreamId));
      }
      let Some(mut sb) = initial_server_buffers.pop() else {
        return Err(crate::Error::http2_reset_stream(
          Http2ErrorCode::RefusedStream,
          Http2Error::NoBuffersForNewStream,
        ));
      };
      let (is_eos, method) = self
        .read_header_and_continuations::<_, false>(&mut sb.lease_mut().rrb, |hf| {
          hf.hsreqh().method.ok_or(crate::Error::HTTP_MissingRequestMethod)
        })
        .await?;
      initial_server_streams.reserve(1);
      initial_server_streams.push_front((method, self.fi.stream_id))?;
      drop(sorp.insert(
        self.fi.stream_id,
        StreamOverallRecvParams {
          body_len: 0,
          has_initial_header: true,
          sb,
          status_code: StatusCode::Ok,
          stream_state: stream_state(is_eos),
          windows: Windows::stream(self.hp, self.hps),
        },
      ));
    }
    Ok(())
  }

  #[inline]
  async fn manage_invalid_stream_state(
    &mut self,
    sorp: &StreamOverallRecvParams<SB>,
  ) -> crate::Result<()> {
    if sorp.stream_state.recv_eos() {
      send_go_away(
        Http2ErrorCode::StreamClosed,
        self.is_conn_open,
        self.last_stream_id,
        self.stream,
      )
      .await;
      return Err(crate::Error::http2_go_away_generic(Http2Error::InvalidReceivedFrameAfterEos));
    }
    Ok(())
  }

  #[inline]
  async fn read_header_and_continuations<H, const IS_TRAILER: bool>(
    &mut self,
    rrb: &mut ReqResBuffer,
    mut headers_cb: impl FnMut(&HeadersFrame<'_>) -> crate::Result<H>,
  ) -> crate::Result<(bool, H)> {
    let (hf, mut hpack_size) = HeadersFrame::read::<IS_CLIENT, IS_TRAILER>(
      self.pfb._current(),
      self.fi,
      &mut rrb.headers,
      self.hp,
      self.hpack_dec,
      &mut rrb.uri,
      self.uri_buffer,
    )?;
    if hf.is_over_size() {
      return Err(crate::Error::http2_go_away_generic(Http2Error::VeryLargeHeadersLen));
    }
    let mut is_eoh = hf.is_eoh();
    let is_eos = hf.is_eos();
    let rslt = headers_cb(&hf)?;
    'continuation_frames: {
      if is_eoh {
        break 'continuation_frames;
      }
      for _ in 0.._max_continuation_frames!() {
        let frame_fi =
          loop_until_some!(read_frame(self.hp, self.is_conn_open, self.pfb, self.stream).await?);
        let has_diff_id = self.fi.stream_id != frame_fi.stream_id;
        let is_not_continuation = frame_fi.ty != FrameInitTy::Continuation;
        if has_diff_id || is_not_continuation {
          return Err(crate::Error::http2_go_away_generic(Http2Error::UnexpectedContinuationFrame));
        }
        let frame_ci = ContinuationFrame::read::<IS_TRAILER>(
          self.pfb._current(),
          frame_fi,
          &mut rrb.headers,
          &mut hpack_size,
          self.hpack_dec,
        )?;
        if frame_ci.is_over_size() {
          return Err(crate::Error::http2_go_away_generic(Http2Error::VeryLargeHeadersLen));
        }
        is_eoh = frame_ci.is_eoh();
        if is_eoh {
          break 'continuation_frames;
        }
      }
      return Err(crate::Error::http2_go_away_generic(
        Http2Error::VeryLargeAmountOfContinuationFrames,
      ));
    }
    Ok((is_eos, rslt))
  }
}
