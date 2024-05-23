use crate::{
  http2::{
    http2_params_send::Http2ParamsSend, FrameInit, FrameInitTy, GoAwayFrame, HpackEncoder,
    Http2Buffer, Http2Error, Http2ErrorCode, Http2Params, PingFrame, ResetStreamFrame,
    SettingsFrame, Sorp, WindowUpdateFrame, Windows, PAD_MASK, U31,
  },
  misc::{PartitionedFilledBuffer, PollOnce, Stream, Usize, _read_until},
};
use core::pin::pin;

#[inline]
pub(crate) fn apply_initial_params<SB>(
  hb: &mut Http2Buffer<SB>,
  hp: &Http2Params,
) -> crate::Result<()> {
  hb.hpack_dec.set_max_bytes(hp.max_hpack_len().0);
  hb.hpack_enc.set_max_dyn_super_bytes(hp.max_hpack_len().1);
  hb.pfb._expand_buffer(*Usize::from(hp.read_buffer_len()));
  Ok(())
}

#[inline]
pub(crate) async fn read_frame<S>(
  hp: &mut Http2Params,
  is_conn_open: &mut bool,
  pfb: &mut PartitionedFilledBuffer,
  stream: &mut S,
) -> crate::Result<Option<FrameInit>>
where
  S: Stream,
{
  if !*is_conn_open {
    return Err(crate::Error::Http2ErrorGoAway(Http2ErrorCode::Cancel, None));
  }
  let mut read = pfb._following_len();
  let buffer = pfb._following_trail_mut();
  for _ in 0.._max_frames_mismatches!() {
    let Some(array) = PollOnce(pin!(_read_until::<9, _>(buffer, &mut read, 0, stream))).await
    else {
      return Ok(None);
    };
    let (opt, data_len) = FrameInit::from_array(array?);
    let Some(fi) = opt else {
      let _ = stream.read(buffer.get_mut(..*Usize::from_u32(data_len)).unwrap_or_default()).await?;
      continue;
    };
    if fi.stream_id.is_not_zero() && fi.is_conn_control() {
      let _ = stream.read(buffer.get_mut(..*Usize::from_u32(data_len)).unwrap_or_default()).await?;
      continue;
    }
    if fi.data_len > hp.max_frame_len() {
      return Err(crate::Error::http2_go_away_generic(Http2Error::LargeArbitraryFrameLen));
    }
    _trace!("Received frame: {fi:?}");
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
      return Err(crate::Error::MISC_UnexpectedBufferState);
    }
    pfb._set_indices(
      pfb._current_end_idx().wrapping_add(9),
      *Usize::from(fi.data_len),
      read.wrapping_sub(*Usize::from(frame_len)),
    )?;
    return Ok(Some(fi));
  }
  Err(crate::Error::http2_go_away_generic(Http2Error::VeryLargeAmountOfFrameMismatches))
}

/// Reads frames and return the first that is NOT related to the connection
#[inline]
pub(crate) async fn read_frame_until<S>(
  conn_windows: &mut Windows,
  hp: &mut Http2Params,
  hpack_enc: &mut HpackEncoder,
  hps: &mut Http2ParamsSend,
  is_conn_open: &mut bool,
  pfb: &mut PartitionedFilledBuffer,
  stream: &mut S,
) -> crate::Result<Option<FrameInit>>
where
  S: Stream,
{
  for _ in 0.._max_frames_mismatches!() {
    pfb._clear_if_following_is_empty();
    let Some(fi) = read_frame(hp, is_conn_open, pfb, stream).await? else {
      return Ok(None);
    };
    if fi.stream_id.is_zero() {
      match fi.ty {
        FrameInitTy::GoAway => {
          let gaf = GoAwayFrame::read(pfb._current(), fi)?;
          return Err(crate::Error::Http2ErrorGoAway(gaf.error_code(), None));
        }
        FrameInitTy::Ping => {
          let mut pf = PingFrame::read(pfb._current(), fi)?;
          if !pf.is_ack() {
            pf.set_ack();
            write_array([&pf.bytes()], *is_conn_open, stream).await?;
          }
        }
        FrameInitTy::Settings => {
          let sf = SettingsFrame::read(pfb._current(), fi)?;
          if !sf.is_ack() {
            hps.update(hpack_enc, &sf, conn_windows)?;
            write_array([SettingsFrame::ack().bytes(&mut [0; 45])], *is_conn_open, stream).await?;
          }
        }
        FrameInitTy::WindowUpdate => {
          let wuf = WindowUpdateFrame::read(pfb._current(), fi)?;
          conn_windows.send.deposit(wuf.size_increment().i32());
        }
        _ => return Err(crate::Error::http2_go_away_generic(Http2Error::FrameIsZeroButShouldNot)),
      }
      continue;
    }
    return Ok(Some(fi));
  }
  Err(crate::Error::http2_go_away_generic(Http2Error::VeryLargeAmountOfFrameMismatches))
}

#[inline]
pub(crate) async fn send_go_away<S>(
  error_code: Http2ErrorCode,
  is_conn_open: &mut bool,
  last_stream_id: U31,
  stream: &mut S,
) where
  S: Stream,
{
  *is_conn_open = false;
  let _rslt = stream.write_all(&GoAwayFrame::new(error_code, last_stream_id).bytes()).await;
}

#[inline]
pub(crate) async fn send_reset_stream<S, SB>(
  error_code: Http2ErrorCode,
  sorp: &mut Sorp<SB>,
  stream_id: U31,
  stream: &mut S,
) -> crate::Result<()>
where
  S: Stream,
{
  if sorp.remove(&stream_id).is_none() {
    return Err(crate::Error::http2_go_away_generic(Http2Error::UnknownStreamReceiver));
  }
  let _rslt = stream.write_all(&ResetStreamFrame::new(error_code, stream_id).bytes()).await;
  Ok(())
}

#[inline]
pub(crate) fn trim_frame_pad(data: &mut &[u8], flags: u8) -> crate::Result<Option<u8>> {
  let mut pad_len = None;
  if flags & PAD_MASK == PAD_MASK {
    let [local_pad_len, rest @ ..] = data else {
      return Err(crate::Error::http2_go_away_generic(Http2Error::InvalidFramePad));
    };
    let diff_opt = rest.len().checked_sub(usize::from(*local_pad_len));
    let Some(local_data) = diff_opt.and_then(|idx| data.get(..idx)) else {
      return Err(crate::Error::http2_go_away_generic(Http2Error::InvalidFramePad));
    };
    *data = local_data;
    pad_len = Some(*local_pad_len);
  }
  Ok(pad_len)
}

#[inline]
pub(crate) async fn write_array<S, const N: usize>(
  array: [&[u8]; N],
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
    for (elem, frame) in rslt.iter_mut().zip(array.iter()) {
      if let ([a, b, c, d, e, f, g, h, i], false) = (frame, is_prev_init) {
        if let (Some(frame_init), _) = FrameInit::from_array([*a, *b, *c, *d, *e, *f, *g, *h, *i]) {
          is_prev_init = true;
          *elem = Some(frame_init);
        }
      } else {
        is_prev_init = false;
      }
    }
    rslt
  });
  stream.write_all_vectored(array).await?;
  Ok(())
}
