use crate::{
  http2::{
    misc::{read_frame_others, read_frame_until, read_frame_until_cb_known_id, reset_stream},
    send_params::SendParams,
    ContinuationFrame, DataFrame, FrameHeaderTy, FrameInit, HeadersFrame, Http2Buffer, Http2Params,
    ReadFrameRslt, ReqResBuffer, StreamState, WindowUpdateFrame, U31,
  },
  misc::{BlocksQueue, Lease, LeaseMut, Stream},
};
use hashbrown::HashMap;

#[derive(Debug)]
pub struct Http2Data<HB, S, const IS_CLIENT: bool> {
  hb: HB,
  hp: Http2Params,
  is_conn_open: bool,
  rapid_resets_num: u8,
  send_params: SendParams,
  stream: S,
  streams_num: u32,
}

impl<HB, S, const IS_CLIENT: bool> Http2Data<HB, S, IS_CLIENT>
where
  HB: LeaseMut<Http2Buffer<IS_CLIENT>>,
  S: Stream,
{
  #[inline]
  pub(crate) fn new(hb: HB, hp: Http2Params, stream: S) -> Self {
    Self {
      hb,
      hp,
      is_conn_open: true,
      rapid_resets_num: 0,
      send_params: SendParams::default(),
      stream,
      streams_num: 0,
    }
  }

  #[inline]
  pub(crate) fn hb_mut(&mut self) -> &mut Http2Buffer<IS_CLIENT> {
    self.hb.lease_mut()
  }

  #[inline]
  pub(crate) fn hp(&self) -> &Http2Params {
    &self.hp
  }

  #[inline]
  pub(crate) fn is_conn_open(&self) -> bool {
    self.is_conn_open
  }

  #[inline]
  pub(crate) fn is_conn_open_and_stream_mut(&mut self) -> (&mut bool, &mut S) {
    (&mut self.is_conn_open, &mut self.stream)
  }

  #[inline]
  pub(crate) fn parts_mut(
    &mut self,
  ) -> (&mut Http2Buffer<IS_CLIENT>, &mut bool, &mut SendParams, &mut S) {
    (self.hb.lease_mut(), &mut self.is_conn_open, &mut self.send_params, &mut self.stream)
  }

  /// Reads a frame that is expected to be the initial header of a message along with its
  /// related continuation frames.
  #[inline]
  pub(crate) async fn read_frames_init<H>(
    &mut self,
    rrb: &mut ReqResBuffer,
    stream_id: U31,
    stream_state: &mut StreamState,
    mut headers_cb: impl FnMut(&HeadersFrame<'_, '_>) -> crate::Result<H>,
    mut read_frame_until_cb: impl FnMut(
      &[u8],
      FrameInit,
      &Http2Params,
      &mut HashMap<U31, BlocksQueue<u8, FrameInit>>,
    ) -> crate::Result<bool>,
  ) -> crate::Result<ReadFrameRslt<ReadFramesInit<H>>> {
    for _ in 0.._max_frames_mismatches!() {
      let Http2Buffer { hpack_dec, hpack_enc, pfb, streams_frames, uri_buffer, .. } =
        self.hb.lease_mut();
      let fi = rfr_resource_or_return!(
        read_frame_until(
          &mut self.hp,
          hpack_enc,
          &mut self.is_conn_open,
          pfb,
          &mut self.send_params,
          &mut self.stream,
          stream_id,
          stream_state,
          &mut self.streams_num,
          |fi, hp, data| { read_frame_until_cb(data, fi, hp, streams_frames) },
          |hp| {
            if self.rapid_resets_num >= hp.max_rapid_resets_num() {
              return Err(crate::Error::ExceedAmountOfRapidResets);
            }
            self.rapid_resets_num = self.rapid_resets_num.wrapping_add(1);
            Ok(())
          }
        )
        .await?
      );
      match fi.ty {
        FrameHeaderTy::Continuation | FrameHeaderTy::Data => {
          return Err(crate::Error::NotAInitialHeaderFrame);
        }
        FrameHeaderTy::Headers => {
          let (hf, mut hpack_size) = HeadersFrame::read::<IS_CLIENT>(
            pfb._current(),
            fi,
            &mut rrb.headers,
            &self.hp,
            hpack_dec,
            &mut rrb.uri,
            uri_buffer,
          )?;
          if hf.is_over_size() {
            return Ok(reset_stream(stream_state, &mut self.streams_num));
          }
          let is_eoh = hf.is_eoh();
          let is_eos = hf.is_eos();
          if is_eos {
            *stream_state = StreamState::HalfClosedRemote;
          }
          let headers_rslt = headers_cb(&hf)?;
          rfr_until_resource!(
            self
              .read_frames_continuation(
                &mut hpack_size,
                (is_eoh, is_eos),
                rrb,
                fi.stream_id,
                stream_state,
              )
              .await?
          );
          return Ok(ReadFrameRslt::Resource(ReadFramesInit {
            headers_rslt,
            hpack_size,
            is_eos,
            stream_id: fi.stream_id,
          }));
        }
        FrameHeaderTy::WindowUpdate => {
          let _wuf = WindowUpdateFrame::read(pfb._current(), fi)?;
        }
        _ => return Err(crate::http2::ErrorCode::ProtocolError.into()),
      }
    }
    Err(crate::Error::VeryLargeAmountOfFrameMismatches)
  }

  /// Reads data and trailer frames that compose a stream.
  #[inline]
  pub(crate) async fn read_frames_others(
    &mut self,
    hpack_size: &mut usize,
    is_eos: bool,
    rrb: &mut ReqResBuffer,
    stream_id: U31,
    stream_state: &mut StreamState,
  ) -> crate::Result<ReadFrameRslt<()>> {
    if is_eos {
      return Ok(ReadFrameRslt::Resource(()));
    }
    let Http2Buffer { hpack_dec, hpack_enc, pfb, streams_frames, uri_buffer, .. } =
      self.hb.lease_mut();
    let mut body_len: u32 = 0;
    let mut counter: u32 = 0;
    loop {
      if counter >= _max_frames_mismatches!() {
        return Err(crate::Error::VeryLargeAmountOfFrameMismatches);
      }
      let (fi, data) = rfr_resource_or_return!(
        read_frame_others(
          &mut self.hp,
          hpack_enc,
          &mut self.is_conn_open,
          pfb,
          &mut self.send_params,
          &mut self.stream,
          stream_id,
          stream_state,
          streams_frames,
          &mut self.streams_num
        )
        .await?
      );
      let check_opt = body_len.checked_add(fi.data_len).filter(|el| *el <= self.hp.max_body_len());
      let Some(local_body_len) = check_opt else {
        return Err(crate::http2::ErrorCode::ProtocolError.into());
      };
      body_len = local_body_len;
      if let FrameHeaderTy::Data = fi.ty {
        let df = DataFrame::read(data, fi)?;
        rrb.data.extend_from_slice(df.data());
        if df.is_eos() {
          *stream_state = StreamState::HalfClosedRemote;
          return Ok(ReadFrameRslt::Resource(()));
        }
      } else {
        let (hf, local_hpack_size) = HeadersFrame::read::<false>(
          data,
          fi,
          &mut rrb.headers,
          &self.hp,
          hpack_dec,
          &mut rrb.uri,
          uri_buffer,
        )?;
        if hf.is_over_size() {
          return Ok(reset_stream(stream_state, &mut self.streams_num));
        }
        *hpack_size = hpack_size.saturating_add(local_hpack_size);
        if hf.is_eoh() {
          return Ok(ReadFrameRslt::Resource(()));
        }
        break;
      }
      counter = counter.wrapping_add(1);
    }

    for _ in 0.._max_continuation_frames!() {
      let (fi, data) = rfr_resource_or_return!(
        read_frame_others(
          &mut self.hp,
          hpack_enc,
          &mut self.is_conn_open,
          pfb,
          &mut self.send_params,
          &mut self.stream,
          stream_id,
          stream_state,
          streams_frames,
          &mut self.streams_num
        )
        .await?
      );
      if ContinuationFrame::read(data, fi, &mut rrb.headers, hpack_size, hpack_dec)?.is_eoh() {
        return Ok(ReadFrameRslt::Resource(()));
      }
      counter = counter.wrapping_add(1);
    }
    Err(crate::Error::VeryLargeAmountOfContinuationFrames)
  }

  /// Reads all header frames, data frames and trailer frames that compose a stream.
  #[inline]
  pub(crate) async fn read_frames_stream<H>(
    &mut self,
    rrb: &mut ReqResBuffer,
    stream_id: U31,
    stream_state: &mut StreamState,
    cb: fn(&HeadersFrame<'_, '_>) -> crate::Result<H>,
  ) -> crate::Result<ReadFrameRslt<H>> {
    let mut rfi = rfr_resource_or_return!(
      self
        .read_frames_init(rrb, stream_id, stream_state, cb, |data, fi, hp, streams_frames| {
          read_frame_until_cb_known_id(data, fi, hp, stream_id, streams_frames)
        })
        .await?
    );
    rfr_resource_or_return!(
      self
        .read_frames_others(&mut rfi.hpack_size, rfi.is_eos, rrb, stream_id, stream_state)
        .await?
    );
    Ok(ReadFrameRslt::Resource(rfi.headers_rslt))
  }

  /// Reads all continuation frames.
  #[inline]
  async fn read_frames_continuation(
    &mut self,
    hpack_size: &mut usize,
    (mut is_eoh, is_eos): (bool, bool),
    rrb: &mut ReqResBuffer,
    stream_id: U31,
    stream_state: &mut StreamState,
  ) -> crate::Result<ReadFrameRslt<()>> {
    if is_eoh || is_eos {
      return Ok(ReadFrameRslt::Resource(()));
    }
    let Http2Buffer { hpack_dec, hpack_enc, pfb, streams_frames, .. } = self.hb.lease_mut();
    for _ in 0.._max_continuation_frames!() {
      let (fi, data) = rfr_until_resource!(
        read_frame_others(
          &mut self.hp,
          hpack_enc,
          &mut self.is_conn_open,
          pfb,
          &mut self.send_params,
          &mut self.stream,
          stream_id,
          stream_state,
          streams_frames,
          &mut self.streams_num
        )
        .await?
      );
      let ci = ContinuationFrame::read(data, fi, &mut rrb.headers, hpack_size, hpack_dec)?;
      is_eoh = ci.is_eoh();
      if is_eoh {
        return Ok(ReadFrameRslt::Resource(()));
      }
    }
    Err(crate::Error::VeryLargeAmountOfContinuationFrames)
  }

  #[inline]
  pub(crate) fn send_params_mut(&mut self) -> &mut SendParams {
    &mut self.send_params
  }

  #[inline]
  pub(crate) fn streams_num_mut(&mut self) -> &mut u32 {
    &mut self.streams_num
  }
}

impl<HB, S, const IS_CLIENT: bool> Lease<Http2Data<HB, S, IS_CLIENT>>
  for Http2Data<HB, S, IS_CLIENT>
{
  #[inline]
  fn lease(&self) -> &Http2Data<HB, S, IS_CLIENT> {
    self
  }
}

impl<HB, S, const IS_CLIENT: bool> LeaseMut<Http2Data<HB, S, IS_CLIENT>>
  for Http2Data<HB, S, IS_CLIENT>
{
  #[inline]
  fn lease_mut(&mut self) -> &mut Http2Data<HB, S, IS_CLIENT> {
    self
  }
}

#[derive(Debug)]
pub(crate) struct ReadFramesInit<H> {
  pub(crate) headers_rslt: H,
  pub(crate) hpack_size: usize,
  pub(crate) is_eos: bool,
  pub(crate) stream_id: U31,
}
