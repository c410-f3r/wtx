use crate::{
  http2::{
    send_params::SendParams, ErrorCode, FrameHeaderTy, FrameInit, GoAwayFrame, HpackEncoder,
    Http2Buffer, Http2Params, PingFrame, ReadFrameRslt, ResetStreamFrame, SettingsFrame,
    StreamState, WindowUpdateFrame, PAD_MASK, U31,
  },
  misc::{
    BlocksQueue, PartitionedFilledBuffer, PollOnce, Stream, Usize, _read_until, _unlikely_elem,
  },
};
use core::pin::pin;
use hashbrown::HashMap;

#[inline]
pub(crate) fn apply_initial_params(hb: &mut Http2Buffer, hp: &Http2Params) -> crate::Result<()> {
  hb.hpack_dec.set_max_bytes(hp.max_cached_headers_len().0);
  hb.hpack_enc.set_max_dyn_super_bytes(hp.max_cached_headers_len().1);
  hb.pfb._expand_buffer(*Usize::from(hp.read_buffer_len()));
  Ok(())
}

#[inline]
pub(crate) fn default_stream_frames() -> BlocksQueue<u8, FrameInit> {
  BlocksQueue::with_capacity(8, 64)
}

#[inline]
pub(crate) async fn read_frame<S>(
  hp: &mut Http2Params,
  is_conn_open: bool,
  pfb: &mut PartitionedFilledBuffer,
  stream: &mut S,
) -> crate::Result<ReadFrameRslt<FrameInit>>
where
  S: Stream,
{
  if !is_conn_open {
    return Ok(ReadFrameRslt::ClosedConnection);
  }
  let mut read = pfb._following_len();
  let buffer = pfb._following_trail_mut();
  let Some(array) = PollOnce(pin!(_read_until::<9, _>(buffer, &mut read, 0, stream))).await else {
    return Ok(ReadFrameRslt::IdleConnection);
  };
  let fi = FrameInit::from_array(array?)?;
  _trace!("Received frame: {fi:?}");
  if fi.data_len > hp.max_frame_len() {
    return Err(crate::Error::VeryLargePayload);
  }
  let frame_len = fi.data_len.wrapping_add(9);
  let mut is_fulfilled = false;
  pfb._expand_following(*Usize::from(fi.data_len));
  for _ in 0..=fi.data_len {
    if read >= *Usize::from(frame_len) {
      is_fulfilled = true;
      break;
    }
    read = read.wrapping_add(
      stream.read(pfb._following_trail_mut().get_mut(read..).unwrap_or_default()).await?,
    );
  }
  if !is_fulfilled {
    return Err(crate::Error::UnexpectedBufferState);
  }
  pfb._set_indices(
    pfb._current_end_idx().wrapping_add(9),
    *Usize::from(fi.data_len),
    read.wrapping_sub(*Usize::from(frame_len)),
  )?;
  Ok(ReadFrameRslt::Resource(fi))
}

/// Reads a non-initial frame that corresponds to the desired `stream_id` which is locally stored
/// or externally reachable.
#[inline]
pub(crate) async fn read_frame_others<'rslt, S>(
  hp: &mut Http2Params,
  hpack_enc: &mut HpackEncoder,
  is_conn_open: &mut bool,
  pfb: &'rslt mut PartitionedFilledBuffer,
  send_params: &mut SendParams,
  stream: &mut S,
  stream_id: U31,
  stream_state: &mut StreamState,
  streams_frames: &'rslt mut HashMap<U31, BlocksQueue<u8, FrameInit>>,
  streams_num: &mut u32,
) -> crate::Result<ReadFrameRslt<(FrameInit, &'rslt [u8])>>
where
  S: Stream,
{
  if let Some(true) = streams_frames.get(&stream_id).map(|el| el.blocks_len() > 0) {
    #[allow(
      // Borrow checker limitation
      clippy::unwrap_used
    )]
    let (fi, data) = streams_frames.get_mut(&stream_id).unwrap().pop_back().unwrap();
    return Ok(ReadFrameRslt::Resource((fi, data)));
  }
  let fi = rfr_resource_or_return!(
    read_frame_until(
      hp,
      hpack_enc,
      is_conn_open,
      pfb,
      send_params,
      stream,
      stream_id,
      stream_state,
      streams_num,
      |fi, local_hp, data| {
        read_frame_until_cb_known_id(data, fi, local_hp, stream_id, streams_frames)
      },
      |_| Ok(()),
    )
    .await?
  );
  let rslt = (fi, pfb._current());
  Ok(ReadFrameRslt::Resource(rslt))
}

/// Fetches a frame until `cb` yields a positive boolean.
#[inline]
pub(crate) async fn read_frame_until<S>(
  hp: &mut Http2Params,
  hpack_enc: &mut HpackEncoder,
  is_conn_open: &mut bool,
  pfb: &mut PartitionedFilledBuffer,
  send_params: &mut SendParams,
  stream: &mut S,
  stream_id: U31,
  stream_state: &mut StreamState,
  streams_num: &mut u32,
  mut loop_cb: impl FnMut(FrameInit, &Http2Params, &[u8]) -> crate::Result<bool>,
  mut reset_cb: impl FnMut(&Http2Params) -> crate::Result<()>,
) -> crate::Result<ReadFrameRslt<FrameInit>>
where
  S: Stream,
{
  for _ in 0.._max_frames_mismatches!() {
    let fi = rfr_resource_or_return!(read_frame(hp, *is_conn_open, pfb, stream).await?);
    if fi.stream_id == U31::ZERO {
      match fi.ty {
        FrameHeaderTy::GoAway => {
          let _ = GoAwayFrame::read(pfb._current(), fi)?;
          let go_away_frame = GoAwayFrame::new(ErrorCode::Cancel, stream_id);
          send_go_away(go_away_frame, (is_conn_open, stream)).await?;
          return _unlikely_elem(Ok(ReadFrameRslt::ClosedConnection));
        }
        FrameHeaderTy::Ping => {
          let mut pf = PingFrame::read(pfb._current(), fi)?;
          if !pf.is_ack() {
            pf.set_ack();
            write_bytes([&pf.bytes()], *is_conn_open, stream).await?;
          }
          continue;
        }
        FrameHeaderTy::Settings => {
          let sf = SettingsFrame::read(pfb._current(), fi)?;
          if !sf.is_ack() {
            send_params.update(hpack_enc, &sf)?;
            write_bytes([SettingsFrame::ack().bytes(&mut [0; 45])], *is_conn_open, stream).await?;
          }
          continue;
        }
        FrameHeaderTy::WindowUpdate => {
          let _wuf = WindowUpdateFrame::read(pfb._current(), fi)?;
          continue;
        }
        _ => return Err(ErrorCode::ProtocolError.into()),
      }
    }
    if let FrameHeaderTy::Reset = fi.ty {
      reset_cb(hp)?;
      let _ = ResetStreamFrame::read(pfb._current(), fi)?;
      return Ok(reset_stream(stream_state, streams_num));
    }
    if loop_cb(fi, hp, pfb._current())? {
      return Ok(ReadFrameRslt::Resource(fi));
    }
    pfb._clear_if_following_is_empty();
  }
  Err(crate::Error::VeryLargeAmountOfFrameMismatches)
}

#[inline]
pub(crate) fn read_frame_until_cb_known_id(
  data: &[u8],
  fi: FrameInit,
  hp: &Http2Params,
  stream_id: U31,
  streams_frames: &mut HashMap<U31, BlocksQueue<u8, FrameInit>>,
) -> crate::Result<bool> {
  if fi.stream_id == stream_id {
    return Ok(true);
  }
  let Some(stream_frames) = streams_frames.get_mut(&fi.stream_id) else {
    return Err(crate::Error::UnknownStreamId);
  };
  if stream_frames.elements_len() > hp.max_buffered_frames_num().into() {
    return Err(crate::Error::VeryLargeAmountOfBufferedFrames);
  }
  stream_frames.push_front([data], fi)?;
  Ok(false)
}

#[inline]
pub(crate) fn read_frame_until_cb_unknown_id(
  data: &[u8],
  fi: FrameInit,
  hp: &Http2Params,
  streams_frames: &mut HashMap<U31, BlocksQueue<u8, FrameInit>>,
) -> crate::Result<bool> {
  let Some(stream_frames) = streams_frames.get_mut(&fi.stream_id) else {
    return Ok(true);
  };
  if stream_frames.elements_len() > hp.max_buffered_frames_num().into() {
    return Err(crate::Error::VeryLargeAmountOfBufferedFrames);
  }
  stream_frames.push_front([data], fi)?;
  Ok(false)
}

#[inline]
pub(crate) fn reset_stream<H>(
  stream_state: &mut StreamState,
  streams_num: &mut u32,
) -> ReadFrameRslt<H> {
  *stream_state = StreamState::Closed;
  *streams_num = streams_num.wrapping_sub(1);
  return ReadFrameRslt::ClosedStream;
}

#[inline]
pub(crate) async fn send_go_away<S>(
  go_away_frame: GoAwayFrame,
  (is_conn_open, stream): (&mut bool, &mut S),
) -> crate::Result<()>
where
  S: Stream,
{
  write_bytes([go_away_frame.bytes().as_slice()], *is_conn_open, stream).await?;
  *is_conn_open = false;
  Ok(())
}

#[inline]
pub(crate) async fn send_reset<S>(
  reset_frame: ResetStreamFrame,
  stream_state: &mut StreamState,
  (is_conn_open, stream): (&mut bool, &mut S),
) -> crate::Result<()>
where
  S: Stream,
{
  *stream_state = StreamState::Closed;
  write_bytes([reset_frame.bytes().as_slice()], *is_conn_open, stream).await?;
  Ok(())
}

#[inline]
pub(crate) fn trim_frame_pad(data: &mut &[u8], flags: u8) -> crate::Result<Option<u8>> {
  let mut pad_len = None;
  if flags & PAD_MASK == PAD_MASK {
    let [local_pad_len, rest @ ..] = data else {
      return _unlikely_elem(Err(ErrorCode::ProtocolError.into()));
    };
    let diff_opt = rest.len().checked_sub(usize::from(*local_pad_len));
    let Some(local_data) = diff_opt.and_then(|idx| data.get(..idx)) else {
      return _unlikely_elem(Err(ErrorCode::ProtocolError.into()));
    };
    *data = local_data;
    pad_len = Some(*local_pad_len);
  }
  Ok(pad_len)
}

#[inline]
pub(crate) async fn write_bytes<S, const N: usize>(
  frames: [&[u8]; N],
  is_conn_open: bool,
  stream: &mut S,
) -> crate::Result<()>
where
  S: Stream,
{
  if !is_conn_open {
    return Ok(());
  }
  _trace!("Sending frame(s): {:?}", {
    let mut is_prev_init = false;
    let mut rslt = [None; N];
    for (elem, frame) in rslt.iter_mut().zip(frames.iter()) {
      if let ([a, b, c, d, e, f, g, h, i], false) = (frame, is_prev_init) {
        if let Ok(frame_init) = FrameInit::from_array([*a, *b, *c, *d, *e, *f, *g, *h, *i]) {
          is_prev_init = true;
          *elem = Some(frame_init);
        }
      } else {
        is_prev_init = false;
      }
    }
    rslt
  });
  stream.write_all_vectored(frames).await?;
  Ok(())
}
