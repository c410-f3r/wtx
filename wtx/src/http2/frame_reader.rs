macro_rules! prft {
  ($fi:expr, $hdpm:ident, $pfb:expr, $stream_reader:expr) => {
    ProcessReceiptFrameTy {
      conn_windows: &mut $hdpm.windows,
      fi: $fi,
      hp: &mut $hdpm.hp,
      hpack_dec: &mut $hdpm.hb.hpack_dec,
      hps: &mut $hdpm.hps,
      is_conn_open: &$hdpm.hb.is_conn_open,
      last_stream_id: &mut $hdpm.last_stream_id,
      pfb: $pfb,
      read_frame_waker: &$hdpm.hb.read_frame_waker,
      recv_streams_num: &mut $hdpm.recv_streams_num,
      stream_reader: $stream_reader,
      stream_writer: &mut $hdpm.stream_writer,
    }
  };
}

use crate::{
  http2::{
    Http2Buffer, Http2Data, Http2Error,
    frame_init::{FrameInit, FrameInitTy},
    go_away_frame::GoAwayFrame,
    misc::{process_higher_operation_err, protocol_err, read_frame, send_go_away, write_array},
    ping_frame::PingFrame,
    process_receipt_frame_ty::ProcessReceiptFrameTy,
    settings_frame::SettingsFrame,
    window_update_frame::WindowUpdateFrame,
  },
  misc::{LeaseMut, net::PartitionedFilledBuffer},
  stream::{StreamReader, StreamWriter},
  sync::{Arc, AtomicBool, AtomicWaker, Lock, RefCounter},
};
use core::{
  future::poll_fn,
  mem,
  pin::pin,
  task::{Poll, ready},
};

pub(crate) async fn frame_reader<HB, HD, SR, SW, const IS_CLIENT: bool>(
  hd: HD,
  is_conn_open: Arc<AtomicBool>,
  max_frame_len: u32,
  mut pfb: PartitionedFilledBuffer,
  read_frame_waker: Arc<AtomicWaker>,
  mut stream_reader: SR,
) where
  HB: LeaseMut<Http2Buffer>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, SW, IS_CLIENT>>,
  SR: StreamReader,
  SW: StreamWriter,
{
  let span = _trace_span!("Starting the reading of frames");
  let _e = span.enter();
  loop {
    let fi = match read_frame::<_, false>(
      &is_conn_open,
      max_frame_len,
      &mut pfb,
      &read_frame_waker,
      &mut stream_reader,
    )
    .await
    {
      Err(err) => {
        process_higher_operation_err(&err, &hd).await;
        finish(Some(err), &hd, &mut pfb).await;
        return;
      }
      Ok(None) => {
        finish(None, &hd, &mut pfb).await;
        return;
      }
      Ok(Some(fi)) => fi,
    };
    if let Err(err) = manage_fi(fi, &hd, &is_conn_open, &mut pfb, &mut stream_reader).await {
      process_higher_operation_err(&err, &hd).await;
      finish(Some(err), &hd, &mut pfb).await;
    }
  }
}

async fn finish<HB, HD, SW, const IS_CLIENT: bool>(
  err: Option<crate::Error>,
  hd: &HD,
  pfb: &mut PartitionedFilledBuffer,
) where
  HB: LeaseMut<Http2Buffer>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, SW, IS_CLIENT>>,
  SW: StreamWriter,
{
  let mut lock = hd.lock().await;
  let hdpm = lock.parts_mut();
  if let Some(elem) = err {
    *hdpm.frame_reader_error = Some(elem);
  }
  mem::swap(pfb, &mut hdpm.hb.pfb);
  _trace!("Finishing the reading of frames");
}

async fn manage_fi<HB, HD, SR, SW, const IS_CLIENT: bool>(
  fi: FrameInit,
  hd: &HD,
  is_conn_open: &AtomicBool,
  pfb: &mut PartitionedFilledBuffer,
  stream_reader: &mut SR,
) -> crate::Result<()>
where
  HB: LeaseMut<Http2Buffer>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, SW, IS_CLIENT>>,
  SR: StreamReader,
  SW: StreamWriter,
{
  match fi.ty {
    FrameInitTy::Continuation => {
      return Err(protocol_err(Http2Error::InvalidContinuationFrame));
    }
    FrameInitTy::Data => {
      let mut lock = hd.lock().await;
      let mut hdpm = lock.parts_mut();
      prft!(fi, hdpm, pfb, stream_reader).data(&mut hdpm.hb.sorp).await?;
    }
    FrameInitTy::GoAway => {
      let gaf = GoAwayFrame::read(pfb.current(), fi)?;
      send_go_away(gaf.error_code(), &mut hd.lock().await.parts_mut()).await;
    }
    FrameInitTy::Headers => {
      let mut lock = hd.lock().await;
      let mut hdpm = lock.parts_mut();
      if hdpm.hb.scrp.contains_key(&fi.stream_id) {
        return Err(protocol_err(Http2Error::UnexpectedNonControlFrame));
      }
      if IS_CLIENT {
        prft!(fi, hdpm, pfb, stream_reader).header_client(&mut hdpm.hb.sorp).await?;
      } else if let Some(elem) = hdpm.hb.sorp.get_mut(&fi.stream_id) {
        prft!(fi, hdpm, pfb, stream_reader).header_server_trailer(elem).await?;
      } else if let Some(ish) = hdpm.hb.initial_server_headers.front_mut() {
        let prft = prft!(fi, hdpm, pfb, stream_reader);
        let rslt = prft.header_server_init(ish, &mut hdpm.hb.sorp).await;
        ish.waker.wake_by_ref();
        hdpm.hb.initial_server_headers.increase_cursor();
        rslt?;
      } else {
        drop(lock);
        let mut lock_pin = pin!(hd.lock());
        poll_fn(|cx| {
          let mut local_lock = lock_pin!(cx, hd, lock_pin);
          let mut local_hdpm = local_lock.parts_mut();
          let Some(ish) = local_hdpm.hb.initial_server_headers.front_mut() else {
            cx.waker().wake_by_ref();
            return Poll::Pending;
          };
          let prft = prft!(fi, local_hdpm, pfb, stream_reader);
          let rslt = ready!(pin!(prft.header_server_init(ish, &mut local_hdpm.hb.sorp)).poll(cx));
          ish.waker.wake_by_ref();
          local_hdpm.hb.initial_server_headers.increase_cursor();
          Poll::Ready(rslt)
        })
        .await?;
      }
    }
    FrameInitTy::Ping => {
      let mut pf = PingFrame::read(pfb.current(), fi)?;
      if !pf.has_ack() {
        pf.set_ack();
        write_array([&pf.bytes()], is_conn_open, hd.lock().await.parts_mut().stream_writer).await?;
      }
    }
    FrameInitTy::Reset => {
      let mut lock = hd.lock().await;
      let mut hdpm = lock.parts_mut();
      let prft = prft!(fi, hdpm, pfb, stream_reader);
      prft.reset(&mut hdpm.hb.scrp, &mut hdpm.hb.sorp).await?;
    }
    FrameInitTy::Settings => {
      let sf = SettingsFrame::read(pfb.current(), fi)?;
      if !sf.has_ack() {
        let mut lock = hd.lock().await;
        let hdpm = lock.parts_mut();
        hdpm.hps.update(&mut hdpm.hb.hpack_enc, &mut hdpm.hb.scrp, &sf, &mut hdpm.hb.sorp)?;
        let array = &mut [0; 45];
        write_array(
          [SettingsFrame::ack().bytes(array)],
          is_conn_open,
          lock.parts_mut().stream_writer,
        )
        .await?;
      }
    }
    FrameInitTy::WindowUpdate => {
      if fi.stream_id.is_zero() {
        let wuf = WindowUpdateFrame::read(pfb.current(), fi)?;
        hd.lock().await.parts_mut().windows.send_mut().deposit(None, wuf.size_increment().i32())?;
      } else {
        let mut lock = hd.lock().await;
        let mut hdpm = lock.parts_mut();
        let prft = prft!(fi, hdpm, pfb, stream_reader);
        prft.window_update(&mut hdpm.hb.scrp, &mut hdpm.hb.sorp)?;
      }
    }
  }
  Ok(())
}
