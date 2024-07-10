//! A computer communications protocol, providing full-duplex communication channels over a single
//! TCP connection.

// # Reading copy
//
// |    Frame   |   With Decompression  | Without Decompression |
// |------------|-----------------------|-----------------------|
// |Single      |(PB -> FB)¹            |(PB -> FB)¹            |
// |Continuation|(PB -> DB)* (DB -> FB)¹|(PB -> FB)*            |

mod close_code;
pub mod compression;
mod frame;
mod frame_buffer;
pub mod handshake;
mod misc;
mod op_code;
mod unmask;
mod web_socket_buffer;
mod web_socket_error;

use crate::{
  misc::{
    from_utf8_basic, from_utf8_ext, CompletionErr, ConnectionState, ExtUtf8Error,
    IncompleteUtf8Char, Lease, LeaseMut, PartitionedFilledBuffer, Stream, Vector, _read_until,
  },
  rng::Rng,
  _MAX_PAYLOAD_LEN,
};
pub use close_code::CloseCode;
pub use compression::{Compression, CompressionLevel, DeflateConfig};
use core::ops::Range;
pub use frame::{
  Frame, FrameControlArray, FrameControlArrayMut, FrameMut, FrameMutControlArray,
  FrameMutControlArrayMut, FrameMutMut, FrameMutVec, FrameMutVecMut, FrameVec, FrameVecMut,
};
pub use frame_buffer::{
  FrameBuffer, FrameBufferControlArray, FrameBufferControlArrayMut, FrameBufferMut, FrameBufferVec,
  FrameBufferVecMut,
};
pub use misc::Expand;
use misc::{define_fb_from_header_params, op_code, FilledBuffer};
pub use op_code::OpCode;
pub(crate) use unmask::unmask;
pub use web_socket_buffer::WebSocketBuffer;
pub use web_socket_error::WebSocketError;

pub(crate) const DFLT_FRAME_BUFFER_VEC_LEN: usize = 32 * 1024;
pub(crate) const MAX_CONTROL_FRAME_LEN: usize = MAX_HDR_LEN_USIZE + MAX_CONTROL_FRAME_PAYLOAD_LEN;
pub(crate) const MAX_CONTROL_FRAME_PAYLOAD_LEN: usize = 125;
pub(crate) const MAX_HDR_LEN_U8: u8 = 14;
pub(crate) const MAX_HDR_LEN_USIZE: usize = 14;
pub(crate) const MIN_HEADER_LEN_USIZE: usize = 2;
pub(crate) const DECOMPRESSION_SUFFIX: &[u8; 4] = &[0, 0, 255, 255];

/// Always masks the payload before sending.
pub type WebSocketClient<NC, RNG, S, WSB> = WebSocket<NC, RNG, S, WSB, true>;
/// [`WebSocketClient`] with a mutable reference of [`WebSocketBuffer`].
pub type WebSocketClientMut<'wsb, NC, RNG, S> =
  WebSocketClient<NC, RNG, S, &'wsb mut WebSocketBuffer>;
/// [`WebSocketClient`] with an owned [`WebSocketBuffer`].
pub type WebSocketClientOwned<NC, RNG, S> = WebSocketClient<NC, RNG, S, WebSocketBuffer>;
/// Always unmasks the payload after receiving.
pub type WebSocketServer<NC, RNG, S, WSB> = WebSocket<NC, RNG, S, WSB, false>;
/// [`WebSocketServer`] with a mutable reference of [`WebSocketBuffer`].
pub type WebSocketServerMut<'wsb, NC, RNG, S> =
  WebSocketServer<NC, RNG, S, &'wsb mut WebSocketBuffer>;
/// [`WebSocketServer`] with an owned [`WebSocketBuffer`].
pub type WebSocketServerOwned<NC, RNG, S> = WebSocketServer<NC, RNG, S, WebSocketBuffer>;

type ReadContinuationFramesCbs<B> = (
  fn(&[u8]) -> crate::Result<Option<IncompleteUtf8Char>>,
  fn(&[u8], &mut Option<IncompleteUtf8Char>) -> crate::Result<()>,
  fn(
    &mut FrameBuffer<B>,
    &ReadFrameInfo,
    usize,
    &mut WebSocketBuffer,
  ) -> crate::Result<(bool, usize)>,
);

/// Protocol implementation over an asynchronous stream.
///
/// <https://tools.ietf.org/html/rfc6455>
#[derive(Debug)]
pub struct WebSocket<NC, RNG, S, WSB, const IS_CLIENT: bool> {
  ct: ConnectionState,
  max_payload_len: usize,
  nc: NC,
  rng: RNG,
  stream: S,
  wsb: WSB,
}

impl<NC, RNG, S, WSB, const IS_CLIENT: bool> WebSocket<NC, RNG, S, WSB, IS_CLIENT> {
  /// Sets whether to automatically close the connection when a received frame payload length
  /// exceeds `max_payload_len`. Defaults to `64 * 1024 * 1024` bytes (64 MiB).
  #[inline]
  pub fn set_max_payload_len(&mut self, max_payload_len: usize) {
    self.max_payload_len = max_payload_len;
  }
}

impl<NC, RNG, S, WSB, const IS_CLIENT: bool> WebSocket<NC, RNG, S, WSB, IS_CLIENT>
where
  NC: compression::NegotiatedCompression,
  RNG: Rng,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  /// Creates a new instance from a stream that supposedly has already completed the handshake.
  #[inline]
  pub fn new(nc: NC, rng: RNG, stream: S, mut wsb: WSB) -> Self {
    wsb.lease_mut().nb._clear_if_following_is_empty();
    wsb.lease_mut().nb._expand_following(MAX_HDR_LEN_USIZE);
    Self { ct: ConnectionState::Open, max_payload_len: _MAX_PAYLOAD_LEN, nc, rng, stream, wsb }
  }

  /// Reads a frame from the stream.
  ///
  /// If a frame is made up of other frames, everything is collected until all fragments are
  /// received.
  #[inline]
  pub async fn read_frame<'fb, B>(
    &mut self,
    fb: &'fb mut FrameBuffer<B>,
  ) -> crate::Result<Frame<&'fb mut FrameBuffer<B>, IS_CLIENT>>
  where
    B: LeaseMut<[u8]> + LeaseMut<Vector<u8>>,
  {
    fb.clear();
    let header_buffer_len = header_placeholder::<IS_CLIENT>();
    let payload_start_idx = header_buffer_len.into();
    let Some(first_rfi) = self.read_first_frame(fb, header_buffer_len, payload_start_idx).await?
    else {
      return Frame::from_fb(fb);
    };
    let mut total_frame_len = payload_start_idx;
    let payload_len = if first_rfi.should_decompress {
      self
        .read_continuation_frames(
          fb,
          &first_rfi,
          payload_start_idx,
          &mut total_frame_len,
          (
            |_| Ok(None),
            |_, _| Ok(()),
            |_, rfi, local_tfl, local_wsb| {
              Ok((true, Self::copy_from_compressed_pb_to_db(local_tfl, rfi, local_wsb)?))
            },
          ),
        )
        .await?;
      let payload_len = Self::copy_from_compressed_db_to_fb(
        &mut self.wsb.lease_mut().db,
        fb,
        &mut self.nc,
        payload_start_idx,
      )?;
      let payload = lease_as_slice(fb.buffer())
        .get(payload_start_idx..payload_start_idx.wrapping_add(payload_len))
        .unwrap_or_default();
      if matches!(first_rfi.op_code, OpCode::Text) && from_utf8_basic(payload).is_err() {
        return Err(crate::Error::MISC_InvalidUTF8);
      }
      payload_len
    } else {
      self
        .read_continuation_frames(
          fb,
          &first_rfi,
          payload_start_idx,
          &mut total_frame_len,
          (
            |curr_payload| {
              Ok(match from_utf8_ext(curr_payload) {
                Err(ExtUtf8Error::Incomplete { incomplete_ending_char, .. }) => {
                  Some(incomplete_ending_char)
                }
                Err(ExtUtf8Error::Invalid { .. }) => {
                  return Err(crate::Error::MISC_InvalidUTF8);
                }
                Ok(_) => None,
              })
            },
            if matches!(first_rfi.op_code, OpCode::Binary) {
              |_, _| Ok(())
            } else {
              Self::manage_continuation_text
            },
            |local_fb, rfi, local_tfl, local_wsb| {
              Ok((
                false,
                Self::copy_from_uncompressed_pb_to_fb(local_fb, local_tfl, &mut local_wsb.nb, rfi)?,
              ))
            },
          ),
        )
        .await?;
      total_frame_len.wrapping_sub(payload_start_idx)
    };
    define_fb_from_header_params::<_, IS_CLIENT>(
      fb,
      true,
      Some(header_buffer_len),
      first_rfi.op_code,
      payload_len,
      self.nc.rsv1(),
    )?;
    Frame::from_fb(fb)
  }

  /// Writes a frame to the stream.
  #[inline]
  pub async fn write_frame<B, FB>(&mut self, frame: &mut Frame<FB, IS_CLIENT>) -> crate::Result<()>
  where
    B: LeaseMut<[u8]>,
    FB: LeaseMut<FrameBuffer<B>>,
  {
    Self::do_write_frame(
      &mut self.ct,
      frame,
      &mut self.nc,
      &mut self.wsb.lease_mut().nb,
      &mut self.rng,
      &mut self.stream,
    )
    .await?;
    Ok(())
  }

  fn begin_fb_bytes_mut<B>(fb: &mut FrameBuffer<B>, payload_start_idx: usize) -> &mut [u8]
  where
    B: LeaseMut<[u8]> + LeaseMut<Vector<u8>>,
  {
    LeaseMut::<[u8]>::lease_mut(fb.buffer_mut()).get_mut(payload_start_idx..).unwrap_or_default()
  }

  fn compress_frame<'pb, B, FB>(
    frame: &Frame<FB, IS_CLIENT>,
    nc: &mut NC,
    pb: &'pb mut PartitionedFilledBuffer,
  ) -> crate::Result<FrameMut<'pb, IS_CLIENT>>
  where
    B: LeaseMut<[u8]>,
    FB: LeaseMut<FrameBuffer<B>>,
  {
    fn expand_pb<'pb, B>(
      len_with_header: usize,
      local_fb: &FrameBuffer<B>,
      local_pb: &'pb mut PartitionedFilledBuffer,
      written: usize,
    ) -> &'pb mut [u8]
    where
      B: Lease<[u8]>,
    {
      let start = len_with_header.wrapping_add(written);
      local_pb._expand_following(start.wrapping_add(local_fb.frame().len()).wrapping_add(128));
      local_pb._following_trail_mut().get_mut(start..).unwrap_or_default()
    }

    let fb = frame.fb().lease();
    let len = pb._following_trail_mut().len();
    let len_with_header = len.wrapping_add(fb.header().len());
    let mut payload_len = nc.compress(
      fb.payload(),
      pb,
      |local_pb| expand_pb(len_with_header, fb, local_pb, 0),
      |local_pb, written| expand_pb(len_with_header, fb, local_pb, written),
    )?;
    if frame.fin() {
      payload_len = payload_len.saturating_sub(4);
    }
    let mut compressed_fb = FrameBufferMut::new(
      pb._following_trail_mut()
        .get_mut(len..len_with_header.wrapping_add(payload_len))
        .unwrap_or_default(),
    );
    define_fb_from_header_params::<_, IS_CLIENT>(
      &mut compressed_fb,
      frame.fin(),
      Some(fb.header_len()),
      frame.op_code(),
      payload_len,
      nc.rsv1(),
    )?;
    FrameMut::from_fb(compressed_fb)
  }

  // Final compressed continuation frame
  fn copy_from_compressed_db_to_fb<B>(
    db: &mut FilledBuffer,
    fb: &mut FrameBuffer<B>,
    nc: &mut NC,
    payload_start_idx: usize,
  ) -> crate::Result<usize>
  where
    B: LeaseMut<[u8]> + LeaseMut<Vector<u8>>,
  {
    db.push_bytes(DECOMPRESSION_SUFFIX)?;
    let mut buffer_len = payload_start_idx
      .checked_add(db.len())
      .map(|element| element.max(lease_as_slice(fb.buffer()).len()));
    let payload_size = nc.decompress(
      db.get(payload_start_idx..).unwrap_or_default(),
      fb,
      |local_fb| Self::begin_fb_bytes_mut(local_fb, payload_start_idx),
      |local_fb, written| Self::expand_fb(&mut buffer_len, local_fb, payload_start_idx, written),
    )?;
    db.clear();
    Ok(payload_size)
  }

  // Intermediate compressed continuation frame
  fn copy_from_compressed_pb_to_db(
    payload_start_idx: usize,
    rfi: &ReadFrameInfo,
    wsb: &mut WebSocketBuffer,
  ) -> crate::Result<usize> {
    Self::copy_from_pb(&mut wsb.db, &mut wsb.nb, rfi, |local_pb, local_db| {
      let n = payload_start_idx.saturating_add(rfi.payload_len);
      local_db.set_idx_through_expansion(n)?;
      local_db
        .get_mut(payload_start_idx..n)
        .unwrap_or_default()
        .copy_from_slice(local_pb._current().get(rfi.header_end_idx..).unwrap_or_default());
      Ok(())
    })?;
    Ok(rfi.payload_len)
  }

  // Final compressed single frame
  fn copy_from_compressed_pb_to_fb<B>(
    fb: &mut FrameBuffer<B>,
    nc: &mut NC,
    payload_start_idx: usize,
    pb: &mut PartitionedFilledBuffer,
    rfi: &ReadFrameInfo,
  ) -> crate::Result<usize>
  where
    B: LeaseMut<[u8]> + LeaseMut<Vector<u8>>,
  {
    let mut buffer_len = payload_start_idx
      .checked_add(rfi.payload_len)
      .map(|element| element.max(lease_as_slice(fb.buffer()).len()));
    let payload_len = Self::copy_from_pb(fb, pb, rfi, |local_pb, local_fb| {
      local_pb._expand_buffer(local_pb._buffer().len().wrapping_add(4));
      let curr_end_idx = local_pb._current().len();
      let curr_end_idx_4p = curr_end_idx.wrapping_add(4);
      let has_following = local_pb._has_following();
      let range = rfi.header_end_idx..curr_end_idx_4p;
      let input = local_pb._current_trail_mut().get_mut(range).unwrap_or_default();
      let orig = if let [.., a, b, c, d] = input {
        let array = [*a, *b, *c, *d];
        *a = 0;
        *b = 0;
        *c = 255;
        *d = 255;
        array
      } else {
        [0, 0, 0, 0]
      };
      if has_following {
        let payload_len = nc.decompress(
          input,
          local_fb,
          |local_local_fb| Self::begin_fb_bytes_mut(local_local_fb, payload_start_idx),
          |local_local_fb, written| {
            Self::expand_fb(&mut buffer_len, local_local_fb, payload_start_idx, written)
          },
        )?;
        if let [.., a, b, c, d] = input {
          *a = orig[0];
          *b = orig[1];
          *c = orig[2];
          *d = orig[3];
        }
        Ok(payload_len)
      } else {
        nc.decompress(
          input,
          local_fb,
          |local_local_fb| Self::begin_fb_bytes_mut(local_local_fb, payload_start_idx),
          |local_local_fb, written| {
            Self::expand_fb(&mut buffer_len, local_local_fb, payload_start_idx, written)
          },
        )
      }
    })?;
    Ok(payload_len)
  }

  fn copy_from_pb<O, T>(
    output: &mut O,
    pb: &mut PartitionedFilledBuffer,
    rfi: &ReadFrameInfo,
    cb: impl FnOnce(&mut PartitionedFilledBuffer, &mut O) -> crate::Result<T>,
  ) -> crate::Result<T> {
    _debug!(
      "{:<5} - {:<5} - {:<25}: {:?}, {:?}",
      <&str>::from(crate::misc::Role::from_is_client(IS_CLIENT)),
      "Read",
      "Masked",
      misc::_truncated_slice(pb._current(), 0..32),
      rfi.op_code
    );

    if !IS_CLIENT {
      unmask(
        pb._current_mut().get_mut(rfi.header_end_idx..).unwrap_or_default(),
        rfi.mask.ok_or(WebSocketError::MissingFrameMask)?,
      );
    }

    _debug!(
      "{:<5} - {:<5} - {:<25}: {:?}, {:?}",
      <&str>::from(crate::misc::Role::from_is_client(IS_CLIENT)),
      "Read",
      "Unmasked",
      misc::_truncated_slice(pb._current(), 0..32),
      rfi.op_code
    );

    let rslt = cb(pb, output)?;
    pb._clear_if_following_is_empty();

    Ok(rslt)
  }

  // Final uncompressed single frame as well as intermediate uncompressed continuation frame
  fn copy_from_uncompressed_pb_to_fb<B>(
    fb: &mut FrameBuffer<B>,
    payload_start_idx: usize,
    pb: &mut PartitionedFilledBuffer,
    rfi: &ReadFrameInfo,
  ) -> crate::Result<usize>
  where
    B: LeaseMut<[u8]> + LeaseMut<Vector<u8>>,
  {
    Self::copy_from_pb(fb, pb, rfi, |local_pb, local_fb| {
      let n = payload_start_idx.saturating_add(rfi.payload_len);
      local_fb.expand_buffer(n)?;
      LeaseMut::<[u8]>::lease_mut(local_fb.buffer_mut())
        .get_mut(payload_start_idx..n)
        .unwrap_or_default()
        .copy_from_slice(local_pb._current().get(rfi.header_end_idx..).unwrap_or_default());
      Ok(())
    })?;
    Ok(rfi.payload_len)
  }

  fn curr_payload_bytes<'bytes, B>(
    db: &'bytes FilledBuffer,
    fb: &'bytes FrameBuffer<B>,
    range: Range<usize>,
    should_use_db: bool,
  ) -> &'bytes [u8]
  where
    B: LeaseMut<[u8]>,
  {
    if should_use_db {
      db.get(range).unwrap_or_default()
    } else {
      fb.buffer().lease().get(range).unwrap_or_default()
    }
  }

  async fn do_write_frame<B, FB>(
    ct: &mut ConnectionState,
    frame: &mut Frame<FB, IS_CLIENT>,
    nc: &mut NC,
    pb: &mut PartitionedFilledBuffer,
    rng: &mut RNG,
    stream: &mut S,
  ) -> crate::Result<()>
  where
    B: LeaseMut<[u8]>,
    FB: LeaseMut<FrameBuffer<B>>,
  {
    let mut should_compress = false;
    if frame.op_code() == OpCode::Close {
      *ct = ConnectionState::Closed;
    }
    if !frame.op_code().is_control() {
      if let Some(first) = frame.fb_mut().lease_mut().header_mut().first_mut() {
        should_compress = nc.rsv1() != 0;
        *first |= nc.rsv1();
      }
    }
    if !should_compress || frame.op_code().is_control() {
      _debug!(
        "{:<5} - {:<5} - {:<25}: {:?}, {:?}",
        <&str>::from(crate::misc::Role::from_is_client(IS_CLIENT)),
        "Write",
        "Unmasked",
        misc::_truncated_slice(frame.fb().lease().frame(), 0..32),
        frame.op_code()
      );
      Self::mask_frame(frame, rng);
      _debug!(
        "{:<5} - {:<5} - {:<25}: {:?}, {:?}",
        <&str>::from(crate::misc::Role::from_is_client(IS_CLIENT)),
        "Write",
        "Masked",
        misc::_truncated_slice(frame.fb().lease().frame(), 0..32),
        frame.op_code()
      );
      stream.write_all(frame.fb().lease().frame()).await?;
    } else {
      _debug!(
        "{:<5} - {:<5} - {:<25}: {:?}, {:?}",
        <&str>::from(crate::misc::Role::from_is_client(IS_CLIENT)),
        "Write",
        "Uncompressed, Unmasked",
        misc::_truncated_slice(frame.fb().lease().frame(), 0..32),
        frame.op_code()
      );
      let mut compressed_frame = Self::compress_frame(frame, nc, pb)?;
      _debug!(
        "{:<5} - {:<5} - {:<25}: {:?}, {:?}",
        <&str>::from(crate::misc::Role::from_is_client(IS_CLIENT)),
        "Write",
        "Compressed, Unmasked",
        misc::_truncated_slice(compressed_frame.fb().frame(), 0..32),
        frame.op_code()
      );
      Self::mask_frame(&mut compressed_frame, rng);
      _debug!(
        "{:<5} - {:<5} - {:<25}: {:?}, {:?}",
        <&str>::from(crate::misc::Role::from_is_client(IS_CLIENT)),
        "Write",
        "Compressed, Masked",
        misc::_truncated_slice(compressed_frame.fb().frame(), 0..32),
        frame.op_code()
      );
      stream.write_all(compressed_frame.fb().frame()).await?;
    };
    Ok(())
  }

  fn expand_fb<'fb, B>(
    buffer_len: &mut Option<usize>,
    fb: &'fb mut FrameBuffer<B>,
    payload_start_idx: usize,
    written: usize,
  ) -> crate::Result<&'fb mut [u8]>
  where
    B: LeaseMut<[u8]> + LeaseMut<Vector<u8>>,
  {
    *buffer_len = buffer_len.and_then(|el| el.checked_mul(15)?.checked_div(10));
    fb.expand_buffer(buffer_len.unwrap_or(usize::MAX))?;
    let start = payload_start_idx.wrapping_add(written);
    Ok(Self::begin_fb_bytes_mut(fb, start))
  }

  async fn fetch_frame_from_stream(&mut self) -> crate::Result<ReadFrameInfo> {
    let mut read = self.wsb.lease_mut().nb._following_len();
    let rfi = Self::fetch_header_from_stream(
      self.max_payload_len,
      &self.nc,
      &mut self.wsb.lease_mut().nb,
      &mut read,
      &mut self.stream,
    )
    .await?;
    if self.ct.is_closed() && rfi.op_code != OpCode::Close {
      return Err(WebSocketError::ConnectionClosed.into());
    }
    Self::fetch_payload_from_stream(
      &mut self.wsb.lease_mut().nb,
      &mut read,
      &rfi,
      &mut self.stream,
    )
    .await?;
    let current_end_idx = self.wsb.lease().nb._current_end_idx();
    self.wsb.lease_mut().nb._set_indices(
      current_end_idx,
      rfi.frame_len,
      read.wrapping_sub(rfi.frame_len),
    )?;
    Ok(rfi)
  }

  async fn fetch_header_from_stream(
    max_payload_len: usize,
    nc: &NC,
    pb: &mut PartitionedFilledBuffer,
    read: &mut usize,
    stream: &mut S,
  ) -> crate::Result<ReadFrameInfo> {
    let buffer = pb._following_trail_mut();

    let first_two = _read_until::<2, S>(buffer, read, 0, stream).await?;

    let rsv1 = first_two[0] & 0b0100_0000;
    let rsv2 = first_two[0] & 0b0010_0000;
    let rsv3 = first_two[0] & 0b0001_0000;

    if rsv2 != 0 || rsv3 != 0 {
      return Err(WebSocketError::InvalidCompressionHeaderParameter.into());
    }

    let should_decompress = if nc.rsv1() == 0 {
      if rsv1 != 0 {
        return Err(WebSocketError::InvalidCompressionHeaderParameter.into());
      }
      false
    } else {
      rsv1 != 0
    };

    let fin = first_two[0] & 0b1000_0000 != 0;
    let length_code = first_two[1] & 0b0111_1111;
    let op_code = op_code(first_two[0])?;

    let (mut header_len, payload_len) = match length_code {
      126 => (4, u16::from_be_bytes(_read_until::<2, S>(buffer, read, 2, stream).await?).into()),
      127 => {
        let payload_len = _read_until::<8, S>(buffer, read, 2, stream).await?;
        (10, u64::from_be_bytes(payload_len).try_into()?)
      }
      _ => (2, length_code.into()),
    };

    let mut mask = None;
    if !IS_CLIENT {
      mask = Some(_read_until::<4, S>(buffer, read, header_len, stream).await?);
      header_len = header_len.wrapping_add(4);
    }

    if op_code.is_control() && !fin {
      return Err(WebSocketError::UnexpectedFragmentedControlFrame.into());
    }
    if op_code == OpCode::Ping && payload_len > MAX_CONTROL_FRAME_PAYLOAD_LEN {
      return Err(WebSocketError::VeryLargeControlFrame.into());
    }
    if payload_len >= max_payload_len {
      return Err(WebSocketError::VeryLargePayload.into());
    }

    Ok(ReadFrameInfo {
      fin,
      frame_len: header_len.wrapping_add(payload_len),
      header_end_idx: header_len,
      mask,
      op_code,
      payload_len,
      should_decompress,
    })
  }

  async fn fetch_payload_from_stream(
    pb: &mut PartitionedFilledBuffer,
    read: &mut usize,
    rfi: &ReadFrameInfo,
    stream: &mut S,
  ) -> crate::Result<()> {
    let mut is_payload_filled = false;
    pb._expand_following(rfi.frame_len);
    for _ in 0..=rfi.frame_len {
      if *read >= rfi.frame_len {
        is_payload_filled = true;
        break;
      }
      *read = read.wrapping_add(
        stream.read(pb._following_trail_mut().get_mut(*read..).unwrap_or_default()).await?,
      );
    }
    if !is_payload_filled {
      return Err(crate::Error::MISC_UnexpectedBufferState);
    }
    Ok(())
  }

  /// If this method returns `false`, then a `ping` frame was received and the caller should fetch
  /// more external data in order to get the desired frame.
  async fn manage_auto_reply(
    ct: &mut ConnectionState,
    curr_payload: &[u8],
    nc: &mut NC,
    op_code: OpCode,
    pb: &mut PartitionedFilledBuffer,
    rng: &mut RNG,
    stream: &mut S,
  ) -> crate::Result<bool> {
    match op_code {
      OpCode::Close if ct.is_open() => {
        match curr_payload {
          [] => {}
          [_] => return Err(WebSocketError::InvalidCloseFrame.into()),
          [a, b, rest @ ..] => {
            let _ = from_utf8_basic(rest)?;
            let is_not_allowed = !CloseCode::try_from(u16::from_be_bytes([*a, *b]))?.is_allowed();
            if is_not_allowed || rest.len() > MAX_CONTROL_FRAME_PAYLOAD_LEN - 2 {
              Self::write_control_frame(
                ct,
                &mut FrameControlArray::close_from_params(
                  CloseCode::Protocol,
                  FrameBuffer::default(),
                  rest,
                )?,
                nc,
                pb,
                rng,
                stream,
              )
              .await?;
              return Err(WebSocketError::InvalidCloseFrame.into());
            }
          }
        }
        Self::write_control_frame(
          ct,
          &mut FrameControlArray::new_fin(FrameBuffer::default(), OpCode::Close, curr_payload)?,
          nc,
          pb,
          rng,
          stream,
        )
        .await?;
        Ok(true)
      }
      OpCode::Ping => {
        Self::write_control_frame(
          ct,
          &mut FrameControlArray::new_fin(FrameBuffer::default(), OpCode::Pong, curr_payload)?,
          nc,
          pb,
          rng,
          stream,
        )
        .await?;
        Ok(false)
      }
      OpCode::Continuation | OpCode::Binary | OpCode::Close | OpCode::Pong | OpCode::Text => {
        Ok(true)
      }
    }
  }

  fn manage_continuation_text(
    curr_payload: &[u8],
    iuc: &mut Option<IncompleteUtf8Char>,
  ) -> crate::Result<()> {
    let tail = if let Some(mut incomplete) = iuc.take() {
      let (rslt, remaining) = incomplete.complete(curr_payload);
      match rslt {
        Err(CompletionErr::HasInvalidBytes) => {
          return Err(crate::Error::MISC_InvalidUTF8);
        }
        Err(CompletionErr::InsufficientInput) => {
          let _ = iuc.replace(incomplete);
          &[]
        }
        Ok(_) => remaining,
      }
    } else {
      curr_payload
    };
    match from_utf8_ext(tail) {
      Err(ExtUtf8Error::Incomplete { incomplete_ending_char, .. }) => {
        *iuc = Some(incomplete_ending_char);
      }
      Err(ExtUtf8Error::Invalid { .. }) => {
        return Err(crate::Error::MISC_InvalidUTF8);
      }
      Ok(_) => {}
    }
    Ok(())
  }

  fn mask_frame<B, FB>(frame: &mut Frame<FB, IS_CLIENT>, rng: &mut RNG)
  where
    B: LeaseMut<[u8]>,
    FB: LeaseMut<FrameBuffer<B>>,
  {
    if IS_CLIENT {
      if let [_, second_byte, .., a, b, c, d] = frame.fb_mut().lease_mut().header_mut() {
        if !has_masked_frame(*second_byte) {
          *second_byte |= 0b1000_0000;
          let mask = rng.u8_4();
          *a = mask[0];
          *b = mask[1];
          *c = mask[2];
          *d = mask[3];
          unmask(frame.fb_mut().lease_mut().payload_mut(), mask);
        }
      }
    }
  }

  async fn read_continuation_frames<B>(
    &mut self,
    fb: &mut FrameBuffer<B>,
    first_rfi: &ReadFrameInfo,
    payload_start_idx: usize,
    total_frame_len: &mut usize,
    (first_text_cb, continuation_cb, copy_cb): ReadContinuationFramesCbs<B>,
  ) -> crate::Result<()>
  where
    B: LeaseMut<[u8]> + LeaseMut<Vector<u8>>,
  {
    let mut iuc = {
      let (should_use_db, payload_len) =
        copy_cb(fb, first_rfi, *total_frame_len, self.wsb.lease_mut())?;
      *total_frame_len = total_frame_len.wrapping_add(payload_len);
      match first_rfi.op_code {
        OpCode::Binary => None,
        OpCode::Text => first_text_cb(Self::curr_payload_bytes(
          &self.wsb.lease().db,
          fb,
          payload_start_idx..*total_frame_len,
          should_use_db,
        ))?,
        OpCode::Close | OpCode::Continuation | OpCode::Ping | OpCode::Pong => {
          return Err(WebSocketError::UnexpectedMessageFrame.into());
        }
      }
    };
    'continuation_frames: loop {
      let (curr_payload, fin, op_code) = 'auto_reply: loop {
        let prev = *total_frame_len;
        let mut rfi = self.fetch_frame_from_stream().await?;
        rfi.should_decompress = first_rfi.should_decompress;
        let (should_use_db, payload_len) =
          copy_cb(fb, &rfi, *total_frame_len, self.wsb.lease_mut())?;
        *total_frame_len = total_frame_len.wrapping_add(payload_len);
        let (db, nb) = self.wsb.lease_mut().parts_mut();
        let curr_payload = Self::curr_payload_bytes(db, fb, prev..*total_frame_len, should_use_db);
        if Self::manage_auto_reply(
          &mut self.ct,
          curr_payload,
          &mut self.nc,
          rfi.op_code,
          nb,
          &mut self.rng,
          &mut self.stream,
        )
        .await?
        {
          break 'auto_reply (curr_payload, rfi.fin, rfi.op_code);
        }
        *total_frame_len = prev;
      };
      match op_code {
        OpCode::Continuation => {
          continuation_cb(curr_payload, &mut iuc)?;
          if fin {
            break 'continuation_frames;
          }
        }
        OpCode::Binary | OpCode::Close | OpCode::Ping | OpCode::Pong | OpCode::Text => {
          return Err(WebSocketError::UnexpectedMessageFrame.into());
        }
      }
    }
    Ok(())
  }

  /// Returns `None` if the frame is single, otherwise, returns the necessary information to
  /// continue fetching from the stream.
  #[inline]
  async fn read_first_frame<B>(
    &mut self,
    fb: &mut FrameBuffer<B>,
    header_buffer_len: u8,
    payload_start_idx: usize,
  ) -> crate::Result<Option<ReadFrameInfo>>
  where
    B: LeaseMut<[u8]> + LeaseMut<Vector<u8>>,
  {
    let first_rfi = 'auto_reply: loop {
      let rfi = self.fetch_frame_from_stream().await?;
      if !rfi.fin {
        break 'auto_reply rfi;
      }
      let pb = &mut self.wsb.lease_mut().nb;
      let payload_len = if rfi.should_decompress {
        Self::copy_from_compressed_pb_to_fb(fb, &mut self.nc, payload_start_idx, pb, &rfi)?
      } else {
        Self::copy_from_uncompressed_pb_to_fb(fb, payload_start_idx, pb, &rfi)?
      };
      define_fb_from_header_params::<_, IS_CLIENT>(
        fb,
        rfi.fin,
        Some(header_buffer_len),
        rfi.op_code,
        payload_len,
        self.nc.rsv1(),
      )?;
      let should_stop = Self::manage_auto_reply(
        &mut self.ct,
        fb.payload(),
        &mut self.nc,
        rfi.op_code,
        &mut self.wsb.lease_mut().nb,
        &mut self.rng,
        &mut self.stream,
      )
      .await?;
      if should_stop {
        match rfi.op_code {
          OpCode::Continuation => {
            return Err(WebSocketError::UnexpectedMessageFrame.into());
          }
          OpCode::Text => {
            let _ = from_utf8_basic(fb.payload())?;
          }
          OpCode::Binary | OpCode::Close | OpCode::Ping | OpCode::Pong => {}
        }
        return Ok(None);
      }
    };
    Ok(Some(first_rfi))
  }

  async fn write_control_frame(
    ct: &mut ConnectionState,
    frame: &mut FrameControlArray<IS_CLIENT>,
    nc: &mut NC,
    pb: &mut PartitionedFilledBuffer,
    rng: &mut RNG,
    stream: &mut S,
  ) -> crate::Result<()> {
    Self::do_write_frame(ct, frame, nc, pb, rng, stream).await?;
    Ok(())
  }
}

/// Parameters of the frame read from a stream
#[derive(Debug)]
struct ReadFrameInfo {
  fin: bool,
  frame_len: usize,
  header_end_idx: usize,
  mask: Option<[u8; 4]>,
  op_code: OpCode,
  payload_len: usize,
  should_decompress: bool,
}

const fn has_masked_frame(second_header_byte: u8) -> bool {
  second_header_byte & 0b1000_0000 != 0
}

const fn header_placeholder<const IS_CLIENT: bool>() -> u8 {
  if IS_CLIENT {
    MAX_HDR_LEN_U8
  } else {
    MAX_HDR_LEN_U8 - 4
  }
}

fn lease_as_slice<T, U>(instance: &T) -> &[U]
where
  T: Lease<[U]>,
{
  instance.lease()
}
