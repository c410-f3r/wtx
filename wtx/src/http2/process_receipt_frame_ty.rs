use crate::{
  http::{HttpError, Method, ReqResBuffer, StatusCode},
  http2::{
    http2_params_send::Http2ParamsSend,
    misc::{
      protocol_err, read_header_and_continuations, send_reset_stream, server_header_stream_state,
    },
    window::WindowsPair,
    DataFrame, FrameInit, HpackDecoder, Http2Error, Http2ErrorCode, Http2Params, ResetStreamFrame,
    Scrp, Sorp, StreamOverallRecvParams, StreamState, UriBuffer, WindowUpdateFrame, Windows, U31,
  },
  misc::{LeaseMut, PartitionedFilledBuffer, StreamReader, StreamWriter, Usize},
};
use alloc::collections::VecDeque;
use core::{marker::PhantomData, sync::atomic::AtomicBool, task::Waker};

#[derive(Debug)]
pub(crate) struct ProcessReceiptFrameTy<'instance, RRB, SR, SW> {
  pub(crate) conn_windows: &'instance mut Windows,
  pub(crate) fi: FrameInit,
  pub(crate) hp: &'instance mut Http2Params,
  pub(crate) hpack_dec: &'instance mut HpackDecoder,
  pub(crate) hps: &'instance mut Http2ParamsSend,
  pub(crate) is_conn_open: &'instance AtomicBool,
  pub(crate) last_stream_id: &'instance mut U31,
  pub(crate) pfb: &'instance mut PartitionedFilledBuffer,
  pub(crate) phantom: PhantomData<RRB>,
  pub(crate) recv_streams_num: &'instance mut u32,
  pub(crate) stream_reader: &'instance mut SR,
  pub(crate) stream_writer: &'instance mut SW,
  pub(crate) uri_buffer: &'instance mut UriBuffer,
}

impl<'instance, RRB, SR, SW> ProcessReceiptFrameTy<'instance, RRB, SR, SW>
where
  RRB: LeaseMut<ReqResBuffer>,
  SR: StreamReader,
  SW: StreamWriter,
{
  #[inline]
  pub(crate) async fn data(self, sorp: &mut Sorp<RRB>) -> crate::Result<()> {
    let Some(elem) = sorp.get_mut(&self.fi.stream_id) else {
      if self.fi.stream_id <= *self.last_stream_id {
        return Err(crate::Error::Http2ErrorGoAway(
          Http2ErrorCode::StreamClosed,
          Some(Http2Error::UnknownDataStreamReceiver),
        ));
      }
      return Err(protocol_err(Http2Error::UnknownDataStreamReceiver));
    };
    if elem.stream_state.recv_eos() {
      return Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::StreamClosed,
        Some(Http2Error::InvalidReceivedFrameAfterEos),
      ));
    }
    let local_body_len_opt = elem.body_len.checked_add(self.fi.data_len);
    let Some(local_body_len) = local_body_len_opt.filter(|el| *el <= self.hp.max_body_len()) else {
      return Err(protocol_err(Http2Error::LargeBodyLen(
        local_body_len_opt,
        self.hp.max_body_len(),
      )));
    };
    elem.body_len = local_body_len;
    let (df, body_bytes) = DataFrame::read(self.pfb._current(), self.fi)?;
    elem.rrb.lease_mut().data.extend_from_slice(body_bytes)?;
    WindowsPair::new(self.conn_windows, &mut elem.windows)
      .withdrawn_recv(
        self.hp,
        self.is_conn_open,
        self.stream_writer,
        self.fi.stream_id,
        df.data_len(),
      )
      .await?;
    if df.has_eos() {
      elem.stream_state = StreamState::HalfClosedRemote;
      elem.waker.wake_by_ref();
    }
    Ok(())
  }

  #[inline]
  pub(crate) async fn header_client(self, sorp: &mut Sorp<RRB>) -> crate::Result<()> {
    let Some(elem) = sorp.get_mut(&self.fi.stream_id) else {
      return Err(protocol_err(Http2Error::UnknownHeaderStreamReceiver));
    };
    let has_eos = if elem.has_initial_header {
      read_header_and_continuations::<_, _, true, true>(
        self.fi,
        self.hp,
        self.hpack_dec,
        self.pfb,
        elem.rrb.lease_mut(),
        self.stream_reader,
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
        self.pfb,
        elem.rrb.lease_mut(),
        self.stream_reader,
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
      elem.waker.wake_by_ref();
    }
    Ok(())
  }

  #[inline]
  pub(crate) async fn header_server_init(
    self,
    initial_server_header_params: &mut VecDeque<(Method, U31)>,
    mut rrb: RRB,
    sorp: &mut Sorp<RRB>,
  ) -> crate::Result<()> {
    if self.fi.stream_id <= *self.last_stream_id || self.fi.stream_id.u32() % 2 == 0 {
      return Err(protocol_err(Http2Error::UnexpectedStreamId));
    }
    if *self.recv_streams_num >= self.hp.max_recv_streams_num() {
      return Err(protocol_err(Http2Error::ExceedAmountOfActiveConcurrentStreams));
    }
    *self.recv_streams_num = self.recv_streams_num.wrapping_add(1);
    *self.last_stream_id = self.fi.stream_id;
    rrb.lease_mut().headers.set_max_bytes(*Usize::from(self.hp.max_headers_len()));
    let (content_length, has_eos, method) = read_header_and_continuations::<_, _, false, false>(
      self.fi,
      self.hp,
      self.hpack_dec,
      self.pfb,
      rrb.lease_mut(),
      self.stream_reader,
      self.uri_buffer,
      |hf| hf.hsreqh().method.ok_or_else(|| HttpError::MissingRequestMethod.into()),
    )
    .await?;
    initial_server_header_params.push_back((method, self.fi.stream_id));
    let stream_state = server_header_stream_state(has_eos);
    drop(sorp.insert(
      self.fi.stream_id,
      StreamOverallRecvParams {
        body_len: 0,
        content_length,
        has_initial_header: true,
        is_stream_open: true,
        rrb,
        status_code: StatusCode::Ok,
        stream_state,
        waker: Waker::noop().clone(),
        windows: Windows::initial(self.hp, self.hps),
      },
    ));
    Ok(())
  }

  #[inline]
  pub(crate) async fn header_server_trailer(
    self,
    sorp: &mut StreamOverallRecvParams<RRB>,
  ) -> crate::Result<()> {
    if sorp.stream_state.recv_eos() {
      return Err(protocol_err(Http2Error::UnexpectedHeaderFrame));
    }
    let (_, has_eos, _) = read_header_and_continuations::<_, _, false, true>(
      self.fi,
      self.hp,
      self.hpack_dec,
      self.pfb,
      sorp.rrb.lease_mut(),
      self.stream_reader,
      self.uri_buffer,
      |_| Ok(()),
    )
    .await?;
    sorp.stream_state = server_header_stream_state(has_eos);
    if has_eos {
      sorp.waker.wake_by_ref();
    }
    Ok(())
  }

  #[inline]
  pub(crate) async fn reset(self, scrp: &mut Scrp, sorp: &mut Sorp<RRB>) -> crate::Result<()> {
    let rsf = ResetStreamFrame::read(self.pfb._current(), self.fi)?;
    if !send_reset_stream(rsf.error_code(), scrp, sorp, self.stream_writer, self.fi.stream_id).await
    {
      return Err(protocol_err(Http2Error::UnknownResetStreamReceiver));
    }
    Ok(())
  }

  #[inline]
  pub(crate) fn window_update(self, scrp: &mut Scrp, sorp: &mut Sorp<RRB>) -> crate::Result<()> {
    if let Some(elem) = scrp.get_mut(&self.fi.stream_id) {
      self.do_window_update(&mut elem.windows, &elem.waker)?;
      return Ok(());
    };
    if let Some(elem) = sorp.get_mut(&self.fi.stream_id) {
      self.do_window_update(&mut elem.windows, &elem.waker)?;
      return Ok(());
    };
    Err(protocol_err(Http2Error::UnknownWindowUpdateStreamReceiver))
  }

  #[inline]
  fn do_window_update(self, windows: &mut Windows, waker: &Waker) -> crate::Result<()> {
    let wuf = WindowUpdateFrame::read(self.pfb._current(), self.fi)?;
    windows.send.deposit(Some(self.fi.stream_id), wuf.size_increment().i32())?;
    waker.wake_by_ref();
    Ok(())
  }
}
