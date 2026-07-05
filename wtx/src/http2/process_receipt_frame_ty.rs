use crate::{
  collections::ArrayVectorCopy,
  http::{HttpError, HttpRecvParams, MsgBufferString, StatusCode, u31::U31},
  http2::{
    Http2Error, Http2ErrorCode, Scorp, Sovrp,
    data_frame::DataFrame,
    frame_init::FrameInit,
    hpack_decoder::HpackDecoder,
    http_send_params::HttpSendParams,
    initial_server_stream_remote::InitialServerStreamRemote,
    misc::{protocol_err, read_header_and_continuations, server_header_stream_state, sorp_mut},
    stream_receiver::StreamOverallRecvParams,
    stream_state::StreamState,
    window::{Windows, WindowsPair},
    window_update_frame::WindowUpdateFrame,
  },
  stream::{BufStreamReader, StreamReader},
  sync::{AtomicU8, AtomicWaker},
};
use core::task::Waker;

#[derive(Debug)]
pub(crate) struct ProcessReceiptFrameTy<'instance, SR> {
  pub(crate) conn_windows: &'instance mut Windows,
  pub(crate) fi: FrameInit,
  pub(crate) hp: &'instance mut HttpRecvParams,
  pub(crate) hpack_dec: &'instance mut HpackDecoder,
  pub(crate) hps: &'instance mut HttpSendParams,
  pub(crate) is_conn_open: &'instance AtomicU8,
  pub(crate) last_stream_id: &'instance mut U31,
  pub(crate) nrb: &'instance mut BufStreamReader,
  pub(crate) read_frame_waker: &'instance AtomicWaker,
  pub(crate) recv_streams_num: &'instance mut u32,
  pub(crate) stream_reader: &'instance mut SR,
}

impl<SR> ProcessReceiptFrameTy<'_, SR>
where
  SR: StreamReader,
{
  pub(crate) fn data(self, sorp: &mut Sovrp) -> crate::Result<ArrayVectorCopy<u8, 26>> {
    let Some(elem) = sorp.get_mut(&self.fi.stream_id) else {
      if self.fi.stream_id <= *self.last_stream_id {
        return Err(crate::Error::Http2ErrorGoAway(
          Http2ErrorCode::StreamClosed,
          Http2Error::UnknownDataStreamReceiver,
        ));
      }
      return Err(protocol_err(Http2Error::UnknownDataStreamReceiver));
    };
    if elem.stream_state.recv_eos() {
      return Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::StreamClosed,
        Http2Error::InvalidReceivedFrameAfterEos,
      ));
    }
    let local_body_len_opt = elem.body_len.checked_add(self.fi.data_len);
    let Some(local_body_len) = local_body_len_opt.filter(|el| *el <= self.hp.max_body_len()) else {
      return Err(protocol_err(Http2Error::LargeBodyLen(local_body_len_opt)));
    };
    elem.body_len = local_body_len;
    let (df, body_bytes) = DataFrame::read(self.nrb.current(), self.fi)?;
    elem.msg_buffer.body.extend_from_copyable_slice(body_bytes)?;
    elem.has_one_or_more_data_frames = true;
    if df.has_eos() {
      elem.stream_state = StreamState::HalfClosedRemote;
    }
    elem.waker.wake_by_ref();
    WindowsPair::new(self.conn_windows, &mut elem.windows).withdrawn_recv(
      self.hp,
      self.fi.stream_id,
      df.data_len(),
    )
  }

  pub(crate) async fn header_client(self, sorp: &mut Sovrp) -> crate::Result<()> {
    let elem = sorp_mut(sorp, self.fi.stream_id)?;
    let has_eos = if elem.has_initial_header {
      read_header_and_continuations::<_, _, true, true>(
        self.fi,
        self.is_conn_open,
        self.hp,
        self.hpack_dec,
        &mut elem.msg_buffer,
        self.nrb,
        self.read_frame_waker,
        self.stream_reader,
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
        &mut elem.msg_buffer,
        self.nrb,
        self.read_frame_waker,
        self.stream_reader,
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

  pub(crate) async fn header_server_init(
    self,
    sorp: &mut Sovrp,
  ) -> crate::Result<InitialServerStreamRemote> {
    if self.fi.stream_id <= *self.last_stream_id || self.fi.stream_id.u32().is_multiple_of(2) {
      return Err(protocol_err(Http2Error::UnexpectedStreamId));
    }
    if *self.recv_streams_num >= self.hp.max_recv_streams_num() {
      return Err(protocol_err(Http2Error::ExceedAmountOfOpenedStreams));
    }
    *self.recv_streams_num = self.recv_streams_num.wrapping_add(1);
    *self.last_stream_id = self.fi.stream_id;
    let mut msg_buffer = MsgBufferString::default();
    let tuple = read_header_and_continuations::<_, _, false, false>(
      self.fi,
      self.is_conn_open,
      self.hp,
      self.hpack_dec,
      &mut msg_buffer,
      self.nrb,
      self.read_frame_waker,
      self.stream_reader,
      |hf| Ok((hf.hsreqh().method.ok_or(HttpError::MissingRequestMethod)?, hf.hsreqh().protocol)),
    )
    .await?;
    let (content_length, has_eos, (method, protocol)) = tuple;
    let stream_state = server_header_stream_state(has_eos);
    drop(sorp.insert(
      self.fi.stream_id,
      StreamOverallRecvParams {
        body_len: 0,
        content_length,
        has_initial_header: true,
        has_one_or_more_data_frames: false,
        is_stream_open: true,
        msg_buffer,
        status_code: StatusCode::Ok,
        stream_state,
        waker: Waker::noop().clone(),
        windows: Windows::initial(self.hp, self.hps),
      },
    ));
    Ok(InitialServerStreamRemote { method, protocol, stream_id: self.fi.stream_id })
  }

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
      &mut sorp.msg_buffer,
      self.nrb,
      self.read_frame_waker,
      self.stream_reader,
      |_| Ok(()),
    )
    .await?;
    sorp.stream_state = server_header_stream_state(has_eos);
    if has_eos {
      sorp.waker.wake_by_ref();
    }
    Ok(())
  }

  pub(crate) fn window_update(self, scorp: &mut Scorp, sovrp: &mut Sovrp) -> crate::Result<()> {
    if let Some(elem) = scorp.get_mut(&self.fi.stream_id) {
      self.do_window_update(&mut elem.windows, &elem.waker)?;
      return Ok(());
    }
    if let Some(elem) = sovrp.get_mut(&self.fi.stream_id) {
      self.do_window_update(&mut elem.windows, &elem.waker)?;
      return Ok(());
    }
    Err(protocol_err(Http2Error::UnknownWindowUpdateStreamReceiver))
  }

  fn do_window_update(self, windows: &mut Windows, waker: &Waker) -> crate::Result<()> {
    let wuf = WindowUpdateFrame::read(self.nrb.current(), self.fi)?;
    windows.send_mut().deposit(Some(self.fi.stream_id), wuf.size_increment().i32())?;
    waker.wake_by_ref();
    Ok(())
  }
}
