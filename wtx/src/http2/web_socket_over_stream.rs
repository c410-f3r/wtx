//! Tools to manage WebSocket connections in HTTP/2 streams

use crate::{
  collection::Vector,
  http::{Headers, StatusCode},
  http2::{
    Http2Buffer, Http2Error, Http2ErrorCode, Http2RecvStatus, ServerStream, misc::protocol_err,
  },
  misc::{ConnectionState, JoinArray, LeaseMut, SingleTypeStorage},
  rng::Xorshift64,
  stream::StreamWriter,
  web_socket::{
    Frame, FrameMut, OpCode, WebSocketReplier,
    read_frame_info::ReadFrameInfo,
    web_socket_reader::{manage_auto_reply, manage_op_code_of_first_final_frame, unmask_nb},
    web_socket_writer::manage_normal_frame,
  },
};

/// WebSocket tunneling
#[derive(Debug)]
pub struct WebSocketOverStream<S> {
  connection_state: ConnectionState,
  no_masking: bool,
  rng: Xorshift64,
  stream: S,
}

impl<HB, S, SW> WebSocketOverStream<S>
where
  HB: LeaseMut<Http2Buffer>,
  S: LeaseMut<ServerStream<HB, SW>> + SingleTypeStorage<Item = (HB, SW)>,
  SW: StreamWriter,
{
  /// Creates a new instance sending an `Ok` status codes that confirms the WebSocket handshake.
  #[inline]
  pub async fn new(
    headers: &Headers,
    no_masking: bool,
    rng: Xorshift64,
    mut stream: S,
  ) -> crate::Result<Self> {
    let hss = stream.lease_mut().common().send_headers(headers, false, StatusCode::Ok).await?;
    if hss.is_closed() {
      return Err(crate::Error::ClosedHttpConnection);
    }
    Ok(Self { connection_state: ConnectionState::Open, no_masking, rng, stream })
  }

  /// Closes the stream as well as the WebSocket connection.
  #[inline]
  pub async fn close(&mut self) -> crate::Result<()> {
    self.write_frame(&mut Frame::new_fin(OpCode::Close, &mut [])).await?;
    self.stream.lease_mut().common().send_reset(Http2ErrorCode::NoError).await;
    Ok(())
  }

  /// Reads a frame from the stream.
  ///
  /// If a frame is made up of other sub-frames or continuations, then everything is collected
  /// until all fragments are received.
  #[inline]
  pub async fn read_frame<'buffer>(
    &mut self,
    buffer: &'buffer mut Vector<u8>,
  ) -> crate::Result<FrameMut<'buffer, false>> {
    buffer.clear();
    let (rfi, is_eos) = recv_data(buffer, self.no_masking, self.stream.lease_mut()).await?;
    if rfi.fin {
      let _is_control_frame = manage_auto_reply::<_, _, true, false>(
        self.stream.lease_mut(),
        &mut self.connection_state,
        self.no_masking,
        rfi.op_code,
        buffer,
        &WebSocketReplier::new(),
        &mut self.rng,
        write_control_frame_cb,
      )
      .await?;
      manage_op_code_of_first_final_frame(rfi.op_code, buffer)?;
      return Ok(FrameMut::new_fin(rfi.op_code, buffer));
    }
    if is_eos {
      return Err(crate::Error::ClosedHttpConnection);
    }
    Err(protocol_err(Http2Error::WebSocketContinuationFrame))
  }

  /// Writes a frame to the stream.
  #[inline]
  pub async fn write_frame<P>(&mut self, frame: &mut Frame<P, false>) -> crate::Result<()>
  where
    P: LeaseMut<[u8]>,
  {
    manage_normal_frame::<_, _, false>(
      &mut self.connection_state,
      frame,
      self.no_masking,
      &mut self.rng,
    );
    let (header, payload) = frame.header_and_payload();
    let common_stream = self.stream.lease_mut().common();
    let [lhs_rslt, rhs_rslt] = JoinArray::new([
      common_stream.send_data(header, false),
      common_stream.send_data(payload.lease(), false),
    ])
    .await;
    if lhs_rslt?.is_closed() || rhs_rslt?.is_closed() {
      return Err(crate::Error::ClosedHttpConnection);
    }
    Ok(())
  }
}

async fn recv_data<HB, SW>(
  buffer: &mut Vector<u8>,
  no_masking: bool,
  stream: &mut ServerStream<HB, SW>,
) -> crate::Result<(ReadFrameInfo, bool)>
where
  HB: LeaseMut<Http2Buffer>,
  SW: StreamWriter,
{
  let (data, is_eos) = match stream.common().recv_data().await? {
    Http2RecvStatus::ClosedConnection => {
      return Err(crate::Error::ClosedHttpConnection);
    }
    Http2RecvStatus::ClosedStream => {
      return Err(crate::Error::ClosedHttpConnection);
    }
    Http2RecvStatus::Eos(data) => (data, true),
    Http2RecvStatus::Ongoing(data) => (data, false),
  };
  let mut slice = data.as_slice();
  let rfi = ReadFrameInfo::from_bytes::<(), false>(&mut slice, usize::MAX, 0, no_masking)?;
  let before = buffer.len();
  buffer.extend_from_copyable_slice(slice)?;
  unmask_nb::<false>(rfi.mask, buffer.get_mut(before..).unwrap_or_default(), no_masking)?;
  Ok((rfi, is_eos))
}

async fn write_control_frame_cb<HB, SW>(
  stream: &mut ServerStream<HB, SW>,
  header: &[u8],
  payload: &[u8],
) -> crate::Result<()>
where
  HB: LeaseMut<Http2Buffer>,
  SW: StreamWriter,
{
  let common_stream = stream.common();
  let [lhs_rslt, rhs_rslt] = JoinArray::new([
    common_stream.send_data(header, false),
    common_stream.send_data(payload, false),
  ])
  .await;
  let _ = lhs_rslt?;
  let _ = rhs_rslt?;
  Ok(())
}
