macro_rules! trace_frame {
  ($this:expr, $span:expr) => {
    let _e = $span._enter();
    _trace!("Receiving frame: {:?}", $this.fi);
  };
}

use crate::{
  http::{Method, StatusCode},
  http2::{
    http2_params_send::Http2ParamsSend,
    misc::{protocol_err, read_frame},
    window::WindowsPair,
    DataFrame, FrameInit, FrameInitTy, HeadersFrame, HpackDecoder, Http2Error, Http2ErrorCode,
    Http2Params, ReqResBuffer, ResetStreamFrame, Scrp, Sorp, StreamBuffer, StreamOverallRecvParams,
    StreamState, UriBuffer, WindowUpdateFrame, Windows, U31,
  },
  misc::{LeaseMut, PartitionedFilledBuffer, Queue, Stream, _Span},
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
  pub(crate) last_stream_id: &'instance mut U31,
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
  pub(crate) async fn data(self, sorp: &mut Sorp<SB>) -> crate::Result<()> {
    let Some(elem) = sorp.get_mut(&self.fi.stream_id) else {
      return Err(protocol_err(Http2Error::UnknownStreamReceiver));
    };
    if elem.stream_state.recv_eos() {
      return Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::StreamClosed,
        Some(Http2Error::InvalidReceivedFrameAfterEos),
      ));
    }
    trace_frame!(self, elem.span);
    let Some(local_body_len) = elem
      .body_len
      .checked_add(self.fi.data_len)
      .filter(|element| *element <= self.hp.max_body_len())
    else {
      return Err(protocol_err(Http2Error::LargeDataFrameLen));
    };
    elem.body_len = local_body_len;
    let (df, data_bytes) = DataFrame::read(self.pfb._current(), self.fi)?;
    elem.sb.lease_mut().rrb.body.reserve(data_bytes.len());
    elem.sb.lease_mut().rrb.body.extend_from_slice(data_bytes)?;
    WindowsPair::new(self.conn_windows, &mut elem.windows)
      .withdrawn_recv(&self.hp, *self.is_conn_open, self.stream, self.fi.stream_id, df.data_len())
      .await?;
    if df.has_eos() {
      elem.stream_state = StreamState::HalfClosedRemote;
    }
    Ok(())
  }

  #[inline]
  pub(crate) async fn header(
    self,
    initial_server_buffers: &mut Vec<SB>,
    initial_server_streams: &mut Queue<(Method, U31)>,
    scrp: &mut Scrp,
    sorp: &mut Sorp<SB>,
  ) -> crate::Result<()> {
    if scrp.contains_key(&self.fi.stream_id) {
      return Err(protocol_err(Http2Error::UnexpectedNonControlFrame));
    }
    if IS_CLIENT {
      self.header_client(sorp).await
    } else {
      self.header_server(initial_server_buffers, initial_server_streams, sorp).await
    }
  }

  #[inline]
  pub(crate) fn reset(self, scrp: &mut Scrp, sorp: &mut Sorp<SB>) -> crate::Result<()> {
    if let Some(elem) = scrp.get_mut(&self.fi.stream_id) {
      return self.do_reset(&elem.span, &mut elem.stream_state);
    };
    if let Some(elem) = sorp.get_mut(&self.fi.stream_id) {
      return self.do_reset(&elem.span, &mut elem.stream_state);
    };
    Err(protocol_err(Http2Error::UnknownStreamReceiver))
  }

  #[inline]
  pub(crate) fn window_update(self, scrp: &mut Scrp, sorp: &mut Sorp<SB>) -> crate::Result<()> {
    if let Some(elem) = scrp.get_mut(&self.fi.stream_id) {
      return self.do_window_update(&elem.span, &mut elem.windows);
    };
    if let Some(elem) = sorp.get_mut(&self.fi.stream_id) {
      return self.do_window_update(&elem.span, &mut elem.windows);
    };
    Err(protocol_err(Http2Error::UnknownStreamReceiver))
  }

  fn do_reset(self, span: &_Span, stream_state: &mut StreamState) -> crate::Result<()> {
    trace_frame!(self, span);
    let rsf = ResetStreamFrame::read(self.pfb._current(), self.fi)?;
    *stream_state = StreamState::Closed;
    Err(crate::Error::Http2ErrorReset(rsf.error_code(), None, self.fi.stream_id.into()))
  }

  fn do_window_update(self, span: &_Span, windows: &mut Windows) -> crate::Result<()> {
    trace_frame!(self, span);
    let wuf = WindowUpdateFrame::read(self.pfb._current(), self.fi)?;
    windows.send.deposit(Some(self.fi.stream_id), wuf.size_increment().i32())?;
    Ok(())
  }

  #[inline]
  async fn header_client(mut self, sorp: &mut Sorp<SB>) -> crate::Result<()> {
    let Some(elem) = sorp.get_mut(&self.fi.stream_id) else {
      return Err(protocol_err(Http2Error::UnknownStreamReceiver));
    };
    trace_frame!(self, elem.span);
    let has_eos = if elem.has_initial_header {
      self
        .read_header_and_continuations::<_, true>(&mut elem.sb.lease_mut().rrb, |_| Ok(()))
        .await?
        .1
    } else {
      let (_, has_eos, status_code) = self
        .read_header_and_continuations::<_, false>(&mut elem.sb.lease_mut().rrb, |hf| {
          hf.hsresh().status_code.ok_or(crate::Error::HTTP_MissingResponseStatusCode)
        })
        .await?;
      elem.has_initial_header = true;
      elem.status_code = status_code;
      has_eos
    };
    if has_eos {
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
    // Trailer
    if let Some(elem) = sorp.get_mut(&self.fi.stream_id) {
      if elem.stream_state.recv_eos() {
        return Err(protocol_err(Http2Error::UnexpectedHeaderFrame));
      }
      let (_, has_eos, _) = self
        .read_header_and_continuations::<_, true>(&mut elem.sb.lease_mut().rrb, |_| Ok(()))
        .await?;
      elem.stream_state = stream_state(has_eos);
    }
    // Initial header
    else {
      if self.fi.stream_id.u32() % 2 == 0 || self.fi.stream_id <= *self.last_stream_id {
        return Err(protocol_err(Http2Error::UnexpectedStreamId));
      }
      *self.last_stream_id = self.fi.stream_id;
      let Some(mut sb) = initial_server_buffers.pop() else {
        return Err(crate::Error::Http2ErrorReset(
          Http2ErrorCode::RefusedStream,
          Some(Http2Error::NoBuffersForNewStream),
          self.fi.stream_id.into(),
        ));
      };

      let (content_length_idx, has_eos, method) = self
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
          content_length_idx,
          has_initial_header: true,
          sb,
          span: _Span::_none(),
          status_code: StatusCode::Ok,
          stream_state: stream_state(has_eos),
          windows: Windows::stream(self.hp, self.hps),
        },
      ));
    }

    Ok(())
  }

  #[inline]
  async fn read_header_and_continuations<H, const IS_TRAILER: bool>(
    &mut self,
    rrb: &mut ReqResBuffer,
    mut headers_cb: impl FnMut(&HeadersFrame<'_>) -> crate::Result<H>,
  ) -> crate::Result<(Option<usize>, bool, H)> {
    if IS_TRAILER && !self.fi.cf.has_eos() {
      return Err(protocol_err(Http2Error::MissingEOSInTrailer));
    }

    if self.fi.cf.has_eoh() {
      let (content_length_idx, hf) = HeadersFrame::read::<IS_CLIENT, IS_TRAILER>(
        self.pfb._current(),
        self.fi,
        &mut rrb.headers,
        self.hp,
        self.hpack_dec,
        &mut rrb.uri,
        self.uri_buffer,
      )?;
      if hf.is_over_size() {
        return Err(protocol_err(Http2Error::VeryLargeHeadersLen));
      }
      return Ok((content_length_idx, hf.has_eos(), headers_cb(&hf)?));
    }

    rrb.body.clear();
    rrb.body.reserve(self.pfb._current().len());
    rrb.body.extend_from_slice(self.pfb._current())?;

    'continuation_frames: {
      for _ in 0.._max_continuation_frames!() {
        let frame_fi = loop_until_some!(
          read_frame::<_, true>(self.hp, self.is_conn_open, self.pfb, self.stream).await?
        );
        let has_diff_id = self.fi.stream_id != frame_fi.stream_id;
        let is_not_continuation = frame_fi.ty != FrameInitTy::Continuation;
        if has_diff_id || is_not_continuation {
          return Err(protocol_err(Http2Error::UnexpectedContinuationFrame));
        }
        rrb.body.reserve(self.pfb._current().len());
        rrb.body.extend_from_slice(self.pfb._current())?;
        if frame_fi.cf.has_eoh() {
          break 'continuation_frames;
        }
      }
      return Err(protocol_err(Http2Error::VeryLargeAmountOfContinuationFrames));
    }

    let (content_length_idx, hf) = HeadersFrame::read::<IS_CLIENT, IS_TRAILER>(
      &rrb.body,
      self.fi,
      &mut rrb.headers,
      self.hp,
      self.hpack_dec,
      &mut rrb.uri,
      self.uri_buffer,
    )?;
    if hf.is_over_size() {
      return Err(protocol_err(Http2Error::VeryLargeHeadersLen));
    }
    rrb.body.clear();
    Ok((content_length_idx, hf.has_eos(), headers_cb(&hf)?))
  }
}
