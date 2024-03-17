use crate::{
  http2::{
    FrameInit, HpackDecoder, Http2Buffer, StreamId, UriBuffer, DEFAULT_MAX_HEADER_LIST_SIZE,
    MAX_FRAME_SIZE_LOWER_BOUND, MAX_FRAME_SIZE_UPPER_BOUND,
  },
  misc::{FnOnceFut, Stream, Usize, _read_until},
};

pub struct Http2Data<S, const IS_CLIENT: bool> {
  hb: Http2Buffer<IS_CLIENT>,
  max_frame_size: u32,
  max_header_list_size: u32,
  stream: S,
}

impl<S, const IS_CLIENT: bool> Http2Data<S, IS_CLIENT>
where
  S: Stream,
{
  #[inline]
  pub(crate) fn new(hb: Http2Buffer<IS_CLIENT>, stream: S) -> Self {
    Self {
      hb,
      max_frame_size: MAX_FRAME_SIZE_LOWER_BOUND,
      max_header_list_size: DEFAULT_MAX_HEADER_LIST_SIZE,
      stream,
    }
  }

  #[inline]
  pub(crate) fn max_frame_size(&self) -> u32 {
    self.max_frame_size
  }

  #[inline]
  pub(crate) fn max_header_list_size(&self) -> u32 {
    self.max_header_list_size
  }

  #[inline]
  pub(crate) fn parts_mut(&mut self) -> (&mut Http2Buffer<IS_CLIENT>, &mut S) {
    (&mut self.hb, &mut self.stream)
  }

  #[inline]
  pub(crate) async fn read_frame<A, R>(
    &mut self,
    aux: A,
    stream_id: StreamId,
    cb: impl for<'any> FnOnceFut<
      (A, &'any mut HpackDecoder, FrameInit, &'any [u8], &'any mut UriBuffer),
      crate::Result<R>,
    >,
  ) -> crate::Result<R> {
    if let Some(elem) = self.hb.streams_data.get_mut(&stream_id) {
      if let Some((fi, data)) = elem.received_frames.pop_front() {
        return Ok(cb((aux, &mut self.hb.hpack_dec, fi, data, &mut self.hb.uri_buffer)).await?);
      }
    }
    let fi = self.do_read_frame().await?;
    cb((aux, &mut self.hb.hpack_dec, fi, self.hb.rb._current(), &mut self.hb.uri_buffer)).await
  }

  #[inline]
  pub(crate) fn set_max_frame_size(&mut self, elem: u32) {
    self.max_frame_size = elem.clamp(MAX_FRAME_SIZE_LOWER_BOUND, MAX_FRAME_SIZE_UPPER_BOUND);
  }

  #[inline]
  pub(crate) fn set_max_header_list_size(&mut self, elem: u32) {
    self.max_header_list_size = elem;
  }

  #[inline]
  pub(crate) fn stream_mut(&mut self) -> &mut S {
    &mut self.stream
  }

  async fn do_read_frame(&mut self) -> crate::Result<FrameInit> {
    let mut read = self.hb.rb._following_len();

    let buffer = self.hb.rb._following_trail_mut();
    let array = _read_until::<9, S>(buffer, &mut read, 0, &mut self.stream).await?;
    let fi = FrameInit::from_array(array)?;

    if fi.len > self.max_frame_size {
      return Err(crate::Error::VeryLargePayload);
    }

    let len = *Usize::from(fi.len);
    let mut is_payload_filled = false;
    self.hb.rb._expand_following(len);
    for _ in 0..fi.len {
      if read >= len {
        is_payload_filled = true;
        break;
      }
      read = read.wrapping_add(
        self
          .stream
          .read(self.hb.rb._following_trail_mut().get_mut(read..).unwrap_or_default())
          .await?,
      );
    }
    if !is_payload_filled {
      return Err(crate::Error::UnexpectedBufferState);
    }

    self.hb.rb._set_indices(self.hb.rb._current_end_idx(), len, read.wrapping_sub(len))?;
    Ok(fi)
  }
}
