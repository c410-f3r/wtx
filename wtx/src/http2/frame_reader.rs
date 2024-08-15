macro_rules! prft {
  ($fi:expr, $hdpm:ident, $is_conn_open:expr, $pfb:expr, $stream_reader:expr) => {
    ProcessReceiptFrameTy {
      conn_windows: &mut $hdpm.windows,
      fi: $fi,
      hp: &mut $hdpm.hp,
      hpack_dec: &mut $hdpm.hb.hpack_dec,
      hps: &mut $hdpm.hps,
      is_conn_open: $is_conn_open,
      last_stream_id: &mut $hdpm.last_stream_id,
      pfb: $pfb,
      phantom: PhantomData,
      recv_streams_num: &mut $hdpm.recv_streams_num,
      stream_reader: $stream_reader,
      stream_writer: &mut $hdpm.stream_writer,
      uri_buffer: &mut $hdpm.hb.uri_buffer,
    }
  };
}

use crate::{
  http::ReqResBuffer,
  http2::{
    misc::{process_higher_operation_err, protocol_err, read_frame_until},
    FrameInitTy, Http2Buffer, Http2Data, Http2Error, IsConnOpenSync, ProcessReceiptFrameTy,
  },
  misc::{
    GenericTime, LeaseMut, Lock, PartitionedFilledBuffer, RefCounter, StreamReader, StreamWriter,
  },
};
use core::{
  future::poll_fn, marker::PhantomData, mem, pin::pin, sync::atomic::AtomicBool, task::Poll,
  time::Duration,
};

pub(crate) async fn frame_reader<HB, HD, RRB, SR, SW, const IS_CLIENT: bool>(
  hd: HD,
  is_conn_open: IsConnOpenSync,
  max_frame_len: u32,
  mut pfb: PartitionedFilledBuffer,
  mut stream_reader: SR,
) -> crate::Result<()>
where
  HB: LeaseMut<Http2Buffer<RRB>>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, RRB, SW, IS_CLIENT>>,
  RRB: LeaseMut<ReqResBuffer>,
  SR: StreamReader,
  SW: StreamWriter,
{
  let span = _trace_span!("Starting the reading of frames");
  let _e = span._enter();
  let mut params: Option<(crate::Error, GenericTime)> = None;
  loop {
    let rslt = if let Some((error, timeout)) = params.take() {
      if timeout.elapsed().map_or(true, |elapsed| elapsed >= Duration::from_millis(300)) {
        finish(&error, &hd, &mut pfb).await;
        return Err(error);
      } else {
        params = Some((error, timeout));
      }
      read(&hd, &is_conn_open, max_frame_len, &mut pfb, &mut stream_reader).await
    } else {
      read(&hd, &is_conn_open, max_frame_len, &mut pfb, &mut stream_reader).await
    };
    match rslt {
      Ok(_) => {}
      Err(rslt_error @ crate::Error::Http2ErrorGoAway(_, None)) => {
        if params.is_none() {
          params = Some((rslt_error, GenericTime::now()))
        }
      }
      Err(rslt_error) => {
        finish(&rslt_error, &hd, &mut pfb).await;
        return Err(rslt_error);
      }
    }
  }
}

#[inline]
async fn read<HB, HD, RRB, SR, SW, const IS_CLIENT: bool>(
  hd: &HD,
  is_conn_open: &AtomicBool,
  max_frame_len: u32,
  mut pfb: &mut PartitionedFilledBuffer,
  stream_reader: &mut SR,
) -> crate::Result<()>
where
  HB: LeaseMut<Http2Buffer<RRB>>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, RRB, SW, IS_CLIENT>>,
  RRB: LeaseMut<ReqResBuffer>,
  SR: StreamReader,
  SW: StreamWriter,
{
  let fi = read_frame_until(hd, is_conn_open, max_frame_len, pfb, stream_reader).await?;
  match fi.ty {
    FrameInitTy::Data => {
      let mut lock = hd.lock().await;
      let mut hdpm = lock.parts_mut();
      prft!(fi, hdpm, &*is_conn_open, &mut pfb, stream_reader).data(&mut hdpm.hb.sorp).await?;
    }
    FrameInitTy::Headers => {
      let mut lock = hd.lock().await;
      let mut hdpm = lock.parts_mut();
      if hdpm.hb.scrp.contains_key(&fi.stream_id) {
        return Err(protocol_err(Http2Error::UnexpectedNonControlFrame));
      }
      if IS_CLIENT {
        prft!(fi, hdpm, &*is_conn_open, &mut pfb, stream_reader)
          .header_client(&mut hdpm.hb.sorp)
          .await?;
      } else {
        if let Some(elem) = hdpm.hb.sorp.get_mut(&fi.stream_id) {
          prft!(fi, hdpm, &*is_conn_open, &mut pfb, stream_reader)
            .header_server_trailer(elem)
            .await?;
        } else {
          if let Some((rrb, waker)) = hdpm.hb.initial_server_header_buffers.pop_front() {
            let rslt = prft!(fi, hdpm, &*is_conn_open, &mut pfb, stream_reader)
              .header_server_init(&mut hdpm.hb.initial_server_header_params, rrb, &mut hdpm.hb.sorp)
              .await;
            waker.wake();
            rslt?;
          } else {
            drop(lock);
            let mut lock_pin = pin!(hd.lock());
            let (mut local_lock, rrb, waker) = poll_fn(|cx| {
              let mut local_lock = lock_pin!(cx, hd, lock_pin);
              let local_hdpm = local_lock.parts_mut();
              let Some((rrb, waker)) = local_hdpm.hb.initial_server_header_buffers.pop_front()
              else {
                cx.waker().wake_by_ref();
                return Poll::Pending;
              };
              Poll::Ready((local_lock, rrb, waker))
            })
            .await;
            let mut local_hdpm = local_lock.parts_mut();
            let rslt = prft!(fi, local_hdpm, &*is_conn_open, &mut pfb, stream_reader)
              .header_server_init(
                &mut local_hdpm.hb.initial_server_header_params,
                rrb,
                &mut local_hdpm.hb.sorp,
              )
              .await;
            waker.wake();
            rslt?;
          }
        }
      }
    }
    FrameInitTy::Reset => {
      let mut lock = hd.lock().await;
      let mut hdpm = lock.parts_mut();
      let prft = prft!(fi, hdpm, &*is_conn_open, &mut pfb, stream_reader);
      prft.reset(&mut hdpm.hb.scrp, &mut hdpm.hb.sorp).await?;
    }
    FrameInitTy::WindowUpdate if fi.stream_id.is_not_zero() => {
      let mut lock = hd.lock().await;
      let mut hdpm = lock.parts_mut();
      let prft = prft!(fi, hdpm, &*is_conn_open, &mut pfb, stream_reader);
      prft.window_update(&mut hdpm.hb.scrp, &mut hdpm.hb.sorp)?;
    }
    _ => {
      return Err(protocol_err(Http2Error::UnexpectedConnFrame));
    }
  }
  Ok(())
}

#[inline]
async fn finish<HB, HD, RRB, SW, const IS_CLIENT: bool>(
  error: &crate::Error,
  hd: &HD,
  pfb: &mut PartitionedFilledBuffer,
) where
  HB: LeaseMut<Http2Buffer<RRB>>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, RRB, SW, IS_CLIENT>>,
  RRB: LeaseMut<ReqResBuffer>,
  SW: StreamWriter,
{
  process_higher_operation_err(error, hd).await;
  mem::swap(pfb, &mut hd.lock().await.parts_mut().hb.pfb);
  _trace!("Finishing the reading of frames");
}
