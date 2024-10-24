//! Tools to manage WebSocket connections in HTTP/2 streams

use crate::{
  http::{Headers, Method, Protocol, StatusCode},
  http2::{Http2Buffer, Http2Data, Http2RecvStatus, SendDataMode, ServerStream},
  misc::{
    ConnectionState, LeaseMut, Lock, RefCounter, SingleTypeStorage, StreamWriter, Vector,
    Xorshift64,
  },
  web_socket::{
    web_socket_reader::{
      manage_auto_reply, manage_op_code_of_continuation_frames,
      manage_op_code_of_first_continuation_frame, manage_op_code_of_first_final_frame,
      manage_text_of_first_continuation_frame, manage_text_of_recurrent_continuation_frames,
      unmask_nb,
    },
    Frame, FrameMut, ReadFrameInfo,
  },
};

/// Verifies if the initial received headers represent a WebSocket connection.
#[inline]
pub fn is_web_socket_handshake(
  headers: &Headers,
  method: Method,
  protocol: Option<Protocol>,
) -> bool {
  method == Method::Connect
    && protocol == Some(Protocol::WebSocket)
    && headers.get_by_name(b"sec-websocket-version").map(|el| el.value) == Some(b"13")
}

/// WebSocket tunneling
#[derive(Debug)]
pub struct WebSocketOverStream<S> {
  connection_state: ConnectionState,
  rng: Xorshift64,
  stream: S,
}

impl<HB, HD, S, SW> WebSocketOverStream<S>
where
  HB: LeaseMut<Http2Buffer>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, SW, false>>,
  S: LeaseMut<ServerStream<HD>> + SingleTypeStorage<Item = HD>,
  SW: StreamWriter,
{
  /// Creates a new instance sending an `Ok` status codes that confirms the WebSocket handshake.
  #[inline]
  pub async fn new(headers: &Headers, rng: Xorshift64, mut stream: S) -> crate::Result<Self> {
    let hss = stream.lease_mut().common().send_headers(headers, false, StatusCode::Ok).await?;
    if hss.is_closed() {
      return Err(crate::Error::ClosedConnection);
    }
    Ok(Self { connection_state: ConnectionState::Open, rng, stream })
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
    let first_rfi = loop {
      let (rfi, is_eos) = recv_data(buffer, self.stream.lease_mut()).await?;
      if !rfi.fin {
        if is_eos {
          return Err(crate::Error::ClosedConnection);
        }
        break rfi;
      }
      if manage_auto_reply::<_, _, false>(
        self.stream.lease_mut(),
        &mut self.connection_state,
        rfi.op_code,
        buffer,
        &mut self.rng,
        &mut write_control_frame_cb,
      )
      .await?
      {
        manage_op_code_of_first_final_frame(rfi.op_code, buffer)?;
        return Ok(FrameMut::new_fin(rfi.op_code, buffer));
      }
    };
    loop {
      let (rfi, is_eos) = recv_data(buffer, self.stream.lease_mut()).await?;
      if !rfi.fin && is_eos {
        return Err(crate::Error::ClosedConnection);
      }
      let begin = buffer.len();
      let mut iuc = manage_op_code_of_first_continuation_frame(
        first_rfi.op_code,
        buffer,
        manage_text_of_first_continuation_frame,
      )?;
      let payload = buffer.get_mut(begin..).unwrap_or_default();
      if !manage_auto_reply::<_, _, false>(
        self.stream.lease_mut(),
        &mut self.connection_state,
        rfi.op_code,
        payload,
        &mut self.rng,
        &mut write_control_frame_cb,
      )
      .await?
      {
        buffer.truncate(begin);
        continue;
      }
      if manage_op_code_of_continuation_frames(
        rfi.fin,
        first_rfi.op_code,
        &mut iuc,
        rfi.op_code,
        payload,
        manage_text_of_recurrent_continuation_frames,
      )? {
        return Ok(FrameMut::new_fin(first_rfi.op_code, buffer));
      }
    }
  }

  /// Writes a frame to the stream.
  #[inline]
  pub async fn write_frame<P>(&mut self, frame: &mut Frame<P, false>) -> crate::Result<()>
  where
    P: LeaseMut<[u8]>,
  {
    let (header, payload) = frame.header_and_payload();
    let hss = self
      .stream
      .lease_mut()
      .common()
      .send_data(SendDataMode::single_data_frame([header, payload.lease()]), false)
      .await?;
    if hss.is_closed() {
      return Err(crate::Error::ClosedConnection);
    }
    Ok(())
  }
}

#[inline]
async fn recv_data<'buffer, HB, HD, SW>(
  buffer: &'buffer mut Vector<u8>,
  stream: &mut ServerStream<HD>,
) -> crate::Result<(ReadFrameInfo, bool)>
where
  HB: LeaseMut<Http2Buffer>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, SW, false>>,
  SW: StreamWriter,
{
  let (data, is_eos) = match stream.common().recv_data().await? {
    Http2RecvStatus::ClosedConnection => {
      return Err(crate::Error::ClosedConnection);
    }
    Http2RecvStatus::ClosedStream => {
      return Err(crate::Error::ClosedConnection);
    }
    Http2RecvStatus::Eos(data) => (data, true),
    Http2RecvStatus::Ongoing(data) => (data, false),
  };
  let mut slice = data.as_slice();
  let rfi = ReadFrameInfo::from_bytes::<_, false>(&mut slice, usize::MAX, &())?;
  let before = buffer.len();
  buffer.extend_from_copyable_slice(slice)?;
  unmask_nb::<false>(buffer.get_mut(before..).unwrap_or_default(), &rfi)?;
  Ok((rfi, is_eos))
}

#[inline]
async fn write_control_frame_cb<HB, HD, SW>(
  stream: &mut ServerStream<HD>,
  header: &[u8],
  payload: &[u8],
) -> crate::Result<()>
where
  HB: LeaseMut<Http2Buffer>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, SW, false>>,
  SW: StreamWriter,
{
  let array = [header, payload];
  let _ = stream.common().send_data(SendDataMode::single_data_frame(array), true).await?;
  Ok(())
}
