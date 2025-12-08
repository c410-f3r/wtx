macro_rules! prft {
  ($fi:expr, $hdpm:ident, $inner:expr, $rd:expr) => {
    ProcessReceiptFrameTy {
      conn_windows: &mut $hdpm.windows,
      fi: $fi,
      hp: &mut $hdpm.hp,
      hpack_dec: &mut $hdpm.hb.hpack_dec,
      hps: &mut $hdpm.hps,
      is_conn_open: &$inner.is_conn_open,
      last_stream_id: &mut $hdpm.last_stream_id,
      rd: $rd,
      read_frame_waker: &$inner.read_frame_waker,
      recv_streams_num: &mut $hdpm.recv_streams_num,
    }
  };
}

use crate::{
  http2::{
    Http2Buffer, Http2Error, Http2Inner,
    frame_init::{FrameInit, FrameInitTy},
    go_away_frame::GoAwayFrame,
    misc::{
      process_higher_operation_err, protocol_err, read_frame, send_go_away, send_reset_stream,
      write_array,
    },
    ping_frame::PingFrame,
    process_receipt_frame_ty::ProcessReceiptFrameTy,
    reader_data::ReaderData,
    reset_stream_frame::ResetStreamFrame,
    settings_frame::SettingsFrame,
    window_update_frame::WindowUpdateFrame,
  },
  misc::LeaseMut,
  stream::{StreamReader, StreamWriter},
  sync::Arc,
};
use core::mem;

pub(crate) async fn frame_reader<HB, SR, SW, const IS_CLIENT: bool>(
  inner: Arc<Http2Inner<HB, SW, IS_CLIENT>>,
  max_frame_len: u32,
  mut rd: ReaderData<SR>,
) where
  HB: LeaseMut<Http2Buffer>,
  SR: StreamReader,
  SW: StreamWriter,
{
  let span = _trace_span!("Starting the reading of frames");
  let _e = span.enter();
  loop {
    let fi = match read_frame::<_, false>(
      &inner.is_conn_open,
      max_frame_len,
      &mut rd,
      &inner.read_frame_waker,
    )
    .await
    {
      Err(err) => {
        process_higher_operation_err(&err, &inner).await;
        finish(Some(err), &inner, &mut rd).await;
        return;
      }
      Ok(None) => {
        finish(None, &inner, &mut rd).await;
        return;
      }
      Ok(Some(fi)) => fi,
    };
    if let Err(err) = manage_fi(fi, &inner, &mut rd).await {
      process_higher_operation_err(&err, &inner).await;
      finish(Some(err), &inner, &mut rd).await;
      return;
    }
  }
}

async fn finish<HB, SR, SW, const IS_CLIENT: bool>(
  err: Option<crate::Error>,
  inner: &Http2Inner<HB, SW, IS_CLIENT>,
  rd: &mut ReaderData<SR>,
) where
  HB: LeaseMut<Http2Buffer>,
  SR: StreamReader,
  SW: StreamWriter,
{
  let mut hd_guard = inner.hd.lock().await;
  let hdpm = hd_guard.parts_mut();
  if let Some(elem) = err {
    *hdpm.frame_reader_error = Some(elem);
  }
  mem::swap(&mut rd.pfb, &mut hdpm.hb.pfb);
  _trace!("Finishing the reading of frames");
}

async fn manage_fi<HB, SR, SW, const IS_CLIENT: bool>(
  fi: FrameInit,
  inner: &Http2Inner<HB, SW, IS_CLIENT>,
  rd: &mut ReaderData<SR>,
) -> crate::Result<()>
where
  HB: LeaseMut<Http2Buffer>,
  SR: StreamReader,
  SW: StreamWriter,
{
  match fi.ty {
    FrameInitTy::Continuation => {
      return Err(protocol_err(Http2Error::InvalidContinuationFrame));
    }
    FrameInitTy::Data => {
      let frame = {
        let mut hd_guard = inner.hd.lock().await;
        let mut hdpm = hd_guard.parts_mut();
        prft!(fi, hdpm, inner, rd).data(&mut hdpm.hb.sorps).await?
      };
      write_array([&frame], &inner.is_conn_open, &mut inner.wd.lock().await.stream_writer).await?;
    }
    FrameInitTy::GoAway => {
      let gaf = GoAwayFrame::read(rd.pfb.current(), fi)?;
      send_go_away(gaf.error_code(), inner).await;
    }
    FrameInitTy::Headers => {
      let mut hd_guard = inner.hd.lock().await;
      let mut hdpm = hd_guard.parts_mut();
      if hdpm.hb.scrps.contains_key(&fi.stream_id) {
        return Err(protocol_err(Http2Error::UnexpectedNonControlFrame));
      }
      if IS_CLIENT {
        prft!(fi, hdpm, inner, rd).header_client(&mut hdpm.hb.sorps).await?;
      } else if let Some(elem) = hdpm.hb.sorps.get_mut(&fi.stream_id) {
        prft!(fi, hdpm, inner, rd).header_server_trailer(elem).await?;
      } else {
        let lss = prft!(fi, hdpm, inner, rd).header_server_init(&mut hdpm.hb.sorps).await?;
        hdpm.hb.initial_server_streams_remote.push_back(lss)?;
        if let Some(elem) = hdpm.hb.initial_server_streams_local.pop_front() {
          elem.wake();
        }
      }
    }
    FrameInitTy::Ping => {
      let mut pf = PingFrame::read(rd.pfb.current(), fi)?;
      if !pf.has_ack() {
        pf.set_ack();
        let stream_writer = &mut inner.wd.lock().await.stream_writer;
        write_array([&pf.bytes()], &inner.is_conn_open, stream_writer).await?;
      }
    }
    FrameInitTy::PushPromise => {
      return Err(protocol_err(Http2Error::PushPromiseIsUnsupported));
    }
    FrameInitTy::Priority => {}
    FrameInitTy::Reset => {
      let rsf = ResetStreamFrame::read(rd.pfb.current(), fi)?;
      if !send_reset_stream(rsf.error_code(), inner, fi.stream_id).await {
        return Err(protocol_err(Http2Error::UnknownResetStreamReceiver));
      }
    }
    FrameInitTy::Settings => {
      let sf = SettingsFrame::read(rd.pfb.current(), fi)?;
      if !sf.has_ack() {
        {
          let mut hd_guard = inner.hd.lock().await;
          let hdpm = hd_guard.parts_mut();
          hdpm.hps.update(&mut hdpm.hb.hpack_enc, &mut hdpm.hb.scrps, &sf, &mut hdpm.hb.sorps)?;
        }
        write_array(
          [SettingsFrame::ack().bytes(&mut [0; 45])],
          &inner.is_conn_open,
          &mut inner.wd.lock().await.stream_writer,
        )
        .await?;
      }
    }
    FrameInitTy::WindowUpdate => {
      let mut hd_guard = inner.hd.lock().await;
      let mut hdpm = hd_guard.parts_mut();
      if fi.stream_id.is_zero() {
        let wuf = WindowUpdateFrame::read(rd.pfb.current(), fi)?;
        hdpm.windows.send_mut().deposit(None, wuf.size_increment().i32())?;
      } else {
        prft!(fi, hdpm, inner, rd).window_update(&mut hdpm.hb.scrps, &mut hdpm.hb.sorps)?;
      }
    }
  }
  Ok(())
}
