//! Tools to manage WebSocket connections in HTTP/2 streams

use crate::{
  collections::{ArrayVectorU8, SingleTypeStorage, Vector},
  futures::JoinArrayVector,
  http::{Headers, StatusCode},
  http2::{Http2Error, Http2ErrorCode, Http2RecvStatus, ServerStream, misc::protocol_err},
  misc::LeaseMut,
  net::StreamWriter,
  rng::Xorshift64,
  tls::{TlsMode, TlsStreamBridge},
  web_socket::{
    Frame, FrameMut, OpCode, WebSocketBridge,
    read_frame::{manage_auto_reply, manage_op_code_of_first_final_frame, unmask_nb},
    read_frame_info::ReadFrameInfo,
    write_frame::mask_frame,
  },
};

/// WebSocket tunneling
#[derive(Debug)]
pub struct WebSocketOverStream<S> {
  no_masking: bool,
  rng: Xorshift64,
  stream: S,
}

impl<S, SW, TM> WebSocketOverStream<S>
where
  S: LeaseMut<ServerStream<SW, TM>> + SingleTypeStorage<Item = (SW, TM)>,
  SW: StreamWriter,
  TM: TlsMode,
{
  /// Creates a new instance sending an `Ok` status codes that confirms the WebSocket handshake.
  #[inline]
  pub async fn new(
    headers: &Headers,
    no_masking: bool,
    rng: Xorshift64,
    mut stream: S,
  ) -> crate::Result<Self> {
    let hss = stream
      .lease_mut()
      .common()
      .send_headers(&mut Vector::new(), headers, false, StatusCode::Ok)
      .await?;
    if hss.is_closed() {
      return Err(crate::Error::ClosedHttpConnection);
    }
    Ok(Self { no_masking, rng, stream })
  }

  /// Closes the stream as well as the WebSocket connection.
  #[inline]
  pub async fn close(&mut self) -> crate::Result<()> {
    self.write_frame(&mut Frame::new(true, OpCode::Close, &mut [], 0)).await?;
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
  ) -> crate::Result<FrameMut<'buffer>> {
    buffer.clear();
    let (rfi, is_eos) = recv_data(buffer, self.no_masking, self.stream.lease_mut()).await?;
    if rfi.fin {
      let _is_control_frame = manage_auto_reply::<_, _, true, false>(
        self.stream.lease_mut(),
        self.no_masking,
        rfi.op_code,
        buffer,
        &mut self.rng,
        &WebSocketBridge::new(TlsStreamBridge::new()),
        write_control_frame_cb,
      )
      .await?;
      manage_op_code_of_first_final_frame(rfi.op_code, buffer)?;
      return Ok(FrameMut::new(true, rfi.op_code, buffer, 0));
    }
    if is_eos {
      return Err(crate::Error::ClosedHttpConnection);
    }
    Err(protocol_err(Http2Error::WebSocketContinuationFrame))
  }

  /// Writes a frame to the stream.
  #[inline]
  pub async fn write_frame<P>(&mut self, frame: &mut Frame<P>) -> crate::Result<()>
  where
    P: LeaseMut<[u8]>,
  {
    mask_frame::<_, _, false>(frame, self.no_masking, &mut self.rng);
    let (header, payload) = frame.header_and_payload();
    let common_stream = self.stream.lease_mut().common();
    let results = JoinArrayVector::new(ArrayVectorU8::<_, 2>::from_array([
      common_stream.send_data(header, false),
      common_stream.send_data(payload.lease(), false),
    ]))
    .await;
    for result in results {
      if result?.is_closed() {
        return Err(crate::Error::ClosedHttpConnection);
      }
    }
    Ok(())
  }
}

fn extend_buffer(
  buffer: &mut Vector<u8>,
  no_masking: bool,
  mut slice: &[u8],
) -> crate::Result<(ReadFrameInfo, usize)> {
  let rfi = ReadFrameInfo::from_bytes::<(), false>(&mut slice, usize::MAX, 0, no_masking)?;
  let before = buffer.len();
  buffer.extend_from_copyable_slice(slice)?;
  Ok((rfi, before))
}

async fn recv_data<SW, TM>(
  buffer: &mut Vector<u8>,
  no_masking: bool,
  stream: &mut ServerStream<SW, TM>,
) -> crate::Result<(ReadFrameInfo, bool)>
where
  SW: StreamWriter,
  TM: TlsMode,
{
  let (before, is_eos, rfi) = match stream
    .common()
    .recv_data(|ongoing_data| extend_buffer(buffer, no_masking, ongoing_data))
    .await?
  {
    Http2RecvStatus::ClosedConnection | Http2RecvStatus::ClosedStream(_) => {
      return Err(crate::Error::ClosedHttpConnection);
    }
    Http2RecvStatus::Eos(data) => {
      let (rfi, before) = extend_buffer(buffer, no_masking, &data)?;
      (before, true, rfi)
    }
    Http2RecvStatus::Ongoing((rfi, before)) => (before, false, rfi),
  };
  unmask_nb::<false>(rfi.mask, buffer.get_mut(before..).unwrap_or_default(), no_masking)?;
  Ok((rfi, is_eos))
}

async fn write_control_frame_cb<SW, TM>(
  stream: &mut ServerStream<SW, TM>,
  header: &[u8],
  payload: &[u8],
) -> crate::Result<()>
where
  SW: StreamWriter,
  TM: TlsMode,
{
  let common_stream = stream.common();
  let results = JoinArrayVector::new(ArrayVectorU8::<_, 2>::from_array([
    common_stream.send_data(header, false),
    common_stream.send_data(payload, false),
  ]))
  .await;
  for result in results {
    if result?.is_closed() {
      return Err(crate::Error::ClosedHttpConnection);
    }
  }
  Ok(())
}
