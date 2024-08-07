macro_rules! trace_frame {
  ($this:expr, $span:expr) => {
    let _e = $span._enter();
    _trace!("Receiving frame: {:?}", $this.fi);
  };
}

use crate::{
  http::{HttpError, ReqResBuffer},
  http2::{
    misc::{protocol_err, read_header_and_continuations, server_header_stream_state},
    window::WindowsPair,
    DataFrame, FrameInit, HpackDecoder, Http2Error, Http2ErrorCode, Http2Params, ResetStreamFrame,
    Scrp, Sorp, StreamState, UriBuffer, WindowUpdateFrame, Windows, U31,
  },
  misc::{LeaseMut, PartitionedFilledBuffer, Stream, _Span},
};
use core::marker::PhantomData;

#[derive(Debug)]
pub(crate) struct ProcessReceiptFrameTy<'instance, RRB, S> {
  pub(crate) conn_windows: &'instance mut Windows,
  pub(crate) fi: FrameInit,
  pub(crate) hp: &'instance mut Http2Params,
  pub(crate) hpack_dec: &'instance mut HpackDecoder,
  pub(crate) is_conn_open: &'instance mut bool,
  pub(crate) last_stream_id: &'instance mut U31,
  pub(crate) pfb: &'instance mut PartitionedFilledBuffer,
  pub(crate) phantom: PhantomData<RRB>,
  pub(crate) recv_streams_num: &'instance mut u32,
  pub(crate) stream: &'instance mut S,
  pub(crate) uri_buffer: &'instance mut UriBuffer,
}

impl<'instance, RRB, S> ProcessReceiptFrameTy<'instance, RRB, S>
where
  RRB: LeaseMut<ReqResBuffer>,
  S: Stream,
{
  #[inline]
  pub(crate) async fn data(self, sorp: &mut Sorp<RRB>) -> crate::Result<()> {
    let Some(elem) = sorp.get_mut(&self.fi.stream_id) else {
      return Err(protocol_err(Http2Error::UnknownDataStreamReceiver));
    };
    if elem.stream_state.recv_eos() {
      return Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::StreamClosed,
        Some(Http2Error::InvalidReceivedFrameAfterEos),
      ));
    }
    trace_frame!(self, elem.span);
    let local_body_len_opt = elem.body_len.checked_add(self.fi.data_len);
    let Some(local_body_len) = local_body_len_opt.filter(|el| *el <= self.hp.max_body_len()) else {
      return Err(protocol_err(Http2Error::LargeBodyLen(
        local_body_len_opt,
        self.hp.max_body_len(),
      )));
    };
    elem.body_len = local_body_len;
    let (df, body_bytes) = DataFrame::read(self.pfb._current(), self.fi)?;
    elem.rrb.lease_mut().extend_body(body_bytes)?;
    WindowsPair::new(self.conn_windows, &mut elem.windows)
      .withdrawn_recv(self.hp, *self.is_conn_open, self.stream, self.fi.stream_id, df.data_len())
      .await?;
    if df.has_eos() {
      elem.stream_state = StreamState::HalfClosedRemote;
    }
    Ok(())
  }

  #[inline]
  pub(crate) async fn header_client(self, sorp: &mut Sorp<RRB>) -> crate::Result<()> {
    let Some(elem) = sorp.get_mut(&self.fi.stream_id) else {
      return Err(protocol_err(Http2Error::UnknownHeaderStreamReceiver));
    };
    trace_frame!(self, elem.span);
    let has_eos = if elem.has_initial_header {
      read_header_and_continuations::<_, _, true, true>(
        self.fi,
        self.hp,
        self.hpack_dec,
        self.is_conn_open,
        self.pfb,
        elem.rrb.lease_mut(),
        self.stream,
        self.uri_buffer,
        |_| Ok(()),
      )
      .await?
      .1
    } else {
      let (_, has_eos, status_code) = read_header_and_continuations::<_, _, true, false>(
        self.fi,
        self.hp,
        self.hpack_dec,
        self.is_conn_open,
        self.pfb,
        elem.rrb.lease_mut(),
        self.stream,
        self.uri_buffer,
        |hf| hf.hsresh().status_code.ok_or_else(|| HttpError::MissingResponseStatusCode.into()),
      )
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
  pub(crate) async fn header_server(
    self,
    initial_server_header: &mut Option<FrameInit>,
    sorp: &mut Sorp<RRB>,
  ) -> crate::Result<()> {
    // Trailer
    if let Some(elem) = sorp.get_mut(&self.fi.stream_id) {
      if elem.stream_state.recv_eos() {
        return Err(protocol_err(Http2Error::UnexpectedHeaderFrame));
      }
      let (_, has_eos, _) = read_header_and_continuations::<_, _, false, true>(
        self.fi,
        self.hp,
        self.hpack_dec,
        self.is_conn_open,
        self.pfb,
        elem.rrb.lease_mut(),
        self.stream,
        self.uri_buffer,
        |_| Ok(()),
      )
      .await?;
      elem.stream_state = server_header_stream_state(has_eos);
    }
    // Initial header
    else {
      if self.fi.stream_id.u32() % 2 == 0 || self.fi.stream_id <= *self.last_stream_id {
        return Err(protocol_err(Http2Error::UnexpectedStreamId));
      }
      if *self.recv_streams_num >= self.hp.max_recv_streams_num() {
        return Err(protocol_err(Http2Error::ExceedAmountOfActiveConcurrentStreams));
      }
      *self.recv_streams_num = self.recv_streams_num.wrapping_add(1);
      *self.last_stream_id = self.fi.stream_id;
      *initial_server_header = Some(self.fi);
    }
    Ok(())
  }

  #[inline]
  pub(crate) fn reset(self, scrp: &mut Scrp, sorp: &mut Sorp<RRB>) -> crate::Result<()> {
    if let Some(elem) = scrp.get_mut(&self.fi.stream_id) {
      return self.do_reset(&elem.span, &mut elem.stream_state);
    };
    if let Some(elem) = sorp.get_mut(&self.fi.stream_id) {
      return self.do_reset(&elem.span, &mut elem.stream_state);
    };
    Err(protocol_err(Http2Error::UnknownResetStreamReceiver))
  }

  #[inline]
  pub(crate) fn window_update(self, scrp: &mut Scrp, sorp: &mut Sorp<RRB>) -> crate::Result<()> {
    if let Some(elem) = scrp.get_mut(&self.fi.stream_id) {
      return self.do_window_update(&elem.span, &mut elem.windows);
    };
    if let Some(elem) = sorp.get_mut(&self.fi.stream_id) {
      return self.do_window_update(&elem.span, &mut elem.windows);
    };
    Err(protocol_err(Http2Error::UnknownWindowUpdateStreamReceiver))
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
}
