use crate::{
  http::{HttpError, StatusCode},
  http2::{
    Http2Error, Http2ErrorCode, Http2Params, Scrp, Sorp,
    data_frame::DataFrame,
    frame_init::FrameInit,
    hpack_decoder::HpackDecoder,
    http2_params_send::Http2ParamsSend,
    initial_server_header::InitialServerHeader,
    misc::{
      protocol_err, read_header_and_continuations, send_reset_stream, server_header_stream_state,
      sorp_mut,
    },
    reset_stream_frame::ResetStreamFrame,
    stream_receiver::StreamOverallRecvParams,
    stream_state::StreamState,
    u31::U31,
    uri_buffer::UriBuffer,
    window::{Windows, WindowsPair},
    window_update_frame::WindowUpdateFrame,
  },
  misc::{StreamReader, StreamWriter, net::PartitionedFilledBuffer},
  sync::{AtomicBool, AtomicWaker},
};
use core::{mem, task::Waker};

#[derive(Debug)]
pub(crate) struct ProcessReceiptFrameTy<'instance, SR, SW> {
  pub(crate) conn_windows: &'instance mut Windows,
  pub(crate) fi: FrameInit,
  pub(crate) hp: &'instance mut Http2Params,
  pub(crate) hpack_dec: &'instance mut HpackDecoder,
  pub(crate) hps: &'instance mut Http2ParamsSend,
  pub(crate) is_conn_open: &'instance AtomicBool,
  pub(crate) last_stream_id: &'instance mut U31,
  pub(crate) pfb: &'instance mut PartitionedFilledBuffer,
  pub(crate) read_frame_waker: &'instance AtomicWaker,
  pub(crate) recv_streams_num: &'instance mut u32,
  pub(crate) stream_reader: &'instance mut SR,
  pub(crate) stream_writer: &'instance mut SW,
  pub(crate) uri_buffer: &'instance mut UriBuffer,
}

impl<SR, SW> ProcessReceiptFrameTy<'_, SR, SW>
where
  SR: StreamReader,
  SW: StreamWriter,
{
  #[inline]
  pub(crate) async fn data(self, sorp: &mut Sorp) -> crate::Result<()> {
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
    elem.rrb.body.extend_from_copyable_slice(body_bytes)?;
    elem.has_one_or_more_data_frames = true;
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
    }
    elem.waker.wake_by_ref();
    Ok(())
  }

  #[inline]
  pub(crate) async fn header_client(self, sorp: &mut Sorp) -> crate::Result<()> {
    let elem = sorp_mut(sorp, self.fi.stream_id)?;
    let has_eos = if elem.has_initial_header {
      read_header_and_continuations::<_, _, true, true>(
        self.fi,
        self.is_conn_open,
        self.hp,
        self.hpack_dec,
        self.pfb,
        self.read_frame_waker,
        &mut elem.rrb,
        self.stream_reader,
        self.uri_buffer,
        |_| Ok(()),
      )
      .await?
      .1
    } else {
      let (_, has_eos, status_code) = read_header_and_continuations::<_, _, true, false>(
        self.fi,
        self.is_conn_open,
        self.hp,
        self.hpack_dec,
        self.pfb,
        self.read_frame_waker,
        &mut elem.rrb,
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
    ish: &mut InitialServerHeader,
    sorp: &mut Sorp,
  ) -> crate::Result<()> {
    if self.fi.stream_id <= *self.last_stream_id || self.fi.stream_id.u32() % 2 == 0 {
      return Err(protocol_err(Http2Error::UnexpectedStreamId));
    }
    if *self.recv_streams_num >= self.hp.max_recv_streams_num() {
      return Err(protocol_err(Http2Error::ExceedAmountOfOpenedStreams));
    }
    *self.recv_streams_num = self.recv_streams_num.wrapping_add(1);
    *self.last_stream_id = self.fi.stream_id;
    let tuple = read_header_and_continuations::<_, _, false, false>(
      self.fi,
      self.is_conn_open,
      self.hp,
      self.hpack_dec,
      self.pfb,
      self.read_frame_waker,
      &mut ish.rrb,
      self.stream_reader,
      self.uri_buffer,
      |hf| Ok((hf.hsreqh().method.ok_or(HttpError::MissingRequestMethod)?, hf.hsreqh().protocol)),
    )
    .await?;
    let (content_length, has_eos, (method, protocol)) = tuple;
    ish.method = method;
    ish.protocol = protocol;
    ish.stream_id = self.fi.stream_id;
    let stream_state = server_header_stream_state(has_eos);
    drop(sorp.insert(
      self.fi.stream_id,
      StreamOverallRecvParams {
        body_len: 0,
        content_length,
        has_initial_header: true,
        has_one_or_more_data_frames: false,
        is_stream_open: true,
        rrb: mem::take(&mut ish.rrb),
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
    sorp: &mut StreamOverallRecvParams,
  ) -> crate::Result<()> {
    if sorp.stream_state.recv_eos() {
      return Err(protocol_err(Http2Error::UnexpectedHeaderFrame));
    }
    let (_, has_eos, _) = read_header_and_continuations::<_, _, false, true>(
      self.fi,
      self.is_conn_open,
      self.hp,
      self.hpack_dec,
      self.pfb,
      self.read_frame_waker,
      &mut sorp.rrb,
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
  pub(crate) async fn reset(self, scrp: &mut Scrp, sorp: &mut Sorp) -> crate::Result<()> {
    let rsf = ResetStreamFrame::read(self.pfb._current(), self.fi)?;
    if !send_reset_stream(rsf.error_code(), scrp, sorp, self.stream_writer, self.fi.stream_id).await
    {
      return Err(protocol_err(Http2Error::UnknownResetStreamReceiver));
    }
    Ok(())
  }

  #[inline]
  pub(crate) fn window_update(self, scrp: &mut Scrp, sorp: &mut Sorp) -> crate::Result<()> {
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
    windows.send_mut().deposit(Some(self.fi.stream_id), wuf.size_increment().i32())?;
    waker.wake_by_ref();
    Ok(())
  }
}
