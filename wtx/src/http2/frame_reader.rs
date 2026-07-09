macro_rules! prft {
  ($fi:expr, $hdpm:ident, $inner:expr, $nrb:expr, $stream_reader:expr) => {
    ProcessReceiptFrameTy {
      conn_windows: &mut $hdpm.windows,
      fi: $fi,
      hp: &mut $hdpm.hp,
      hpack_dec: &mut $hdpm.hb.hpack_dec,
      hps: &mut $hdpm.hps,
      last_stream_id: &mut $hdpm.last_stream_id,
      nrb: $nrb,
      recv_streams_num: &mut $hdpm.recv_streams_num,
      stream_reader: $stream_reader,
    }
  };
}

use crate::{
  http2::{
    Http2Error, Http2Inner,
    frame_init::{FrameInit, FrameInitTy},
    go_away_frame::GoAwayFrame,
    misc::{
      manage_termination, process_higher_operation_err, protocol_err, read_frame,
      send_reset_stream, write_array,
    },
    ping_frame::PingFrame,
    process_receipt_frame_ty::ProcessReceiptFrameTy,
    reset_stream_frame::ResetStreamFrame,
    settings_frame::SettingsFrame,
    window_update_frame::WindowUpdateFrame,
  },
  stream::{BufStreamReader, StreamReader, StreamWriter},
  sync::Arc,
  tls::{TlsMode, TlsStreamBridge, TlsStreamReader},
};
use core::{future::poll_fn, hint::cold_path, mem, pin::pin, task::Poll};

pub(crate) async fn frame_reader<SR, SW, TM, const IS_CLIENT: bool>(
  inner: Arc<Http2Inner<SW, TM, IS_CLIENT>>,
  max_frame_len: u32,
  mut nrb: BufStreamReader,
  stream_bridge: TlsStreamBridge<IS_CLIENT>,
  mut stream_reader: TlsStreamReader<SR, TM, IS_CLIENT>,
) where
  SR: StreamReader,
  SW: StreamWriter,
  TM: TlsMode,
{
  let span = _trace_span!("Starting the reading of frames");
  let _e = span.enter();
  let mut tls_fut = pin!(stream_bridge.listen());

  loop {
    if TM::TY.is_plain_text() {
      let rslt = read_frame::<_, false>(max_frame_len, &mut nrb, &mut stream_reader).await;
      if !manage_iteration(&inner, &mut nrb, rslt, &mut stream_reader).await {
        break;
      }
      continue;
    }

    let http2 = {
      let mut http2_fut = pin!(read_frame::<_, false>(max_frame_len, &mut nrb, &mut stream_reader));
      'inner: loop {
        let (http2_opt, tls_opt) = poll_fn(|cx| {
          let http2_poll = http2_fut.as_mut().poll(cx);
          let tls_poll = tls_fut.as_mut().poll(cx);
          match (http2_poll, tls_poll) {
            (Poll::Ready(http2), Poll::Ready(tls)) => {
              cold_path();
              Poll::Ready((Some(http2), Some(tls)))
            }
            (Poll::Ready(http2), Poll::Pending) => Poll::Ready((Some(http2), None)),
            (Poll::Pending, Poll::Ready(tls)) => {
              cold_path();
              Poll::Ready((None, Some(tls)))
            }
            (Poll::Pending, Poll::Pending) => {
              cold_path();
              Poll::Pending
            }
          }
        })
        .await;
        if let Some(data) = tls_opt {
          // It is not necessary to close the connection here because such state will be propagated
          // to the HTTP/2 reader.
          cold_path();
          let _rslt = inner.wd.lock().await.manage_bridge_data(data).await;
          tls_fut.set(stream_bridge.listen());
        }
        if let Some(http2) = http2_opt {
          break 'inner http2;
        }
      }
    };
    if !manage_iteration(&inner, &mut nrb, http2, &mut stream_reader).await {
      break;
    }
  }
}

async fn finish<SW, TM, const IS_CLIENT: bool>(
  err: Option<crate::Error>,
  inner: &Http2Inner<SW, TM, IS_CLIENT>,
  nrb: &mut BufStreamReader,
) where
  SW: StreamWriter,
{
  let mut hd_guard = inner.hd.lock().await;
  let hdpm = hd_guard.parts_mut();
  if let Some(elem) = err {
    *hdpm.frame_reader_error = Some(elem);
  }
  mem::swap(nrb, &mut hdpm.hb.nrb);
  _trace!("Finishing the reading of frames");
}

// Returns `false` if the connection should be closed.
async fn manage_iteration<SR, SW, TM, const IS_CLIENT: bool>(
  inner: &Http2Inner<SW, TM, IS_CLIENT>,
  nrb: &mut BufStreamReader,
  rslt: crate::Result<Option<FrameInit>>,
  stream_reader: &mut TlsStreamReader<SR, TM, IS_CLIENT>,
) -> bool
where
  SR: StreamReader,
  SW: StreamWriter,
  TM: TlsMode,
{
  let fi = match rslt {
    Err(err) => {
      process_higher_operation_err(&err, inner).await;
      finish(Some(err), inner, nrb).await;
      return false;
    }
    Ok(None) => {
      finish(None, inner, nrb).await;
      return false;
    }
    Ok(Some(fi)) => fi,
  };
  if let Err(err) = manage_fi(fi, inner, nrb, stream_reader).await {
    process_higher_operation_err(&err, inner).await;
    finish(Some(err), inner, nrb).await;
    return false;
  }
  true
}

async fn manage_fi<SR, SW, TM, const IS_CLIENT: bool>(
  fi: FrameInit,
  inner: &Http2Inner<SW, TM, IS_CLIENT>,
  nrb: &mut BufStreamReader,
  stream_reader: &mut SR,
) -> crate::Result<()>
where
  SR: StreamReader,
  SW: StreamWriter,
  TM: TlsMode,
{
  match fi.ty {
    FrameInitTy::Continuation => {
      return Err(protocol_err(Http2Error::InvalidContinuationFrame));
    }
    FrameInitTy::Data => {
      let frame = {
        let mut hd_guard = inner.hd.lock().await;
        let mut hdpm = hd_guard.parts_mut();
        prft!(fi, hdpm, inner, nrb, stream_reader).data(&mut hdpm.hb.sorps)?
      };
      write_array([&frame], &mut *inner.wd.lock().await).await?;
    }
    FrameInitTy::GoAway => {
      let gaf = GoAwayFrame::read(nrb.current(), fi)?;
      manage_termination::<_, _, _, true>(gaf.error_code(), inner).await;
    }
    FrameInitTy::Headers => {
      let mut hd_guard = inner.hd.lock().await;
      let mut hdpm = hd_guard.parts_mut();
      if hdpm.hb.scrps.contains_key(&fi.stream_id) {
        return Err(protocol_err(Http2Error::UnexpectedNonControlFrame));
      }
      if IS_CLIENT {
        prft!(fi, hdpm, inner, nrb, stream_reader).header_client(&mut hdpm.hb.sorps).await?;
      } else if let Some(elem) = hdpm.hb.sorps.get_mut(&fi.stream_id) {
        prft!(fi, hdpm, inner, nrb, stream_reader).header_server_trailer(elem).await?;
      } else {
        let lss =
          prft!(fi, hdpm, inner, nrb, stream_reader).header_server_init(&mut hdpm.hb.sorps).await?;
        hdpm.hb.initial_server_streams_remote.push_back(lss)?;
        if let Some(elem) = hdpm.hb.initial_server_streams_local.pop_front() {
          elem.wake();
        }
      }
    }
    FrameInitTy::Ping => {
      let mut pf = PingFrame::read(nrb.current(), fi)?;
      if !pf.has_ack() {
        pf.set_ack();
        write_array([&pf.bytes()], &mut *inner.wd.lock().await).await?;
      }
    }
    FrameInitTy::PushPromise => {
      return Err(protocol_err(Http2Error::PushPromiseIsUnsupported));
    }
    FrameInitTy::Priority => {}
    FrameInitTy::Reset => {
      let rsf = ResetStreamFrame::read(nrb.current(), fi)?;
      if !send_reset_stream(rsf.error_code(), inner, fi.stream_id).await {
        return Err(protocol_err(Http2Error::UnknownResetStreamReceiver));
      }
    }
    FrameInitTy::Settings => {
      let sf = SettingsFrame::read(nrb.current(), fi)?;
      if !sf.has_ack() {
        {
          let mut hd_guard = inner.hd.lock().await;
          let hdpm = hd_guard.parts_mut();
          hdpm.hps.update(&mut hdpm.hb.hpack_enc, &mut hdpm.hb.scrps, &sf, &mut hdpm.hb.sorps)?;
        }
        let buffer = &mut [0; 45];
        write_array([SettingsFrame::ack().bytes(buffer)], &mut *inner.wd.lock().await).await?;
      }
    }
    FrameInitTy::WindowUpdate => {
      let mut hd_guard = inner.hd.lock().await;
      let mut hdpm = hd_guard.parts_mut();
      if fi.stream_id.is_zero() {
        let wuf = WindowUpdateFrame::read(nrb.current(), fi)?;
        hdpm.windows.send_mut().deposit(None, wuf.size_increment().i32())?;
      } else {
        prft!(fi, hdpm, inner, nrb, stream_reader)
          .window_update(&mut hdpm.hb.scrps, &mut hdpm.hb.sorps)?;
      }
    }
  }
  Ok(())
}
