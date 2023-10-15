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
mod mask;
mod misc;
mod op_code;
mod web_socket_error;
#[cfg(feature = "tracing")]
use crate::role::Role;

use crate::{
  buffer::Buffer,
  misc::{from_utf8_ext_rslt, from_utf8_opt, CompleteErr, ExtUtf8Error, IncompleteUtf8Char},
  rng::Rng,
  web_socket::{
    compression::NegotiatedCompression,
    misc::{define_fb_from_header_params, header_placeholder, op_code},
  },
  PartitionedBuffer, Stream, MAX_PAYLOAD_LEN,
};
use alloc::vec::Vec;
pub use close_code::CloseCode;
pub use compression::{Compression, CompressionLevel, DeflateConfig};
use core::{borrow::BorrowMut, ops::Range};
pub use frame::{
  Frame, FrameControlArray, FrameControlArrayMut, FrameMut, FrameMutControlArray,
  FrameMutControlArrayMut, FrameMutMut, FrameMutVec, FrameMutVecMut, FrameVec, FrameVecMut,
};
pub use frame_buffer::{
  FrameBuffer, FrameBufferControlArray, FrameBufferControlArrayMut, FrameBufferMut, FrameBufferVec,
  FrameBufferVecMut,
};
pub use mask::unmask;
pub use op_code::OpCode;
pub use web_socket_error::WebSocketError;

pub(crate) const DFLT_FRAME_BUFFER_VEC_LEN: usize = 32 * 1024;
pub(crate) const MAX_CONTROL_FRAME_LEN: usize = MAX_HDR_LEN_USIZE + MAX_CONTROL_FRAME_PAYLOAD_LEN;
pub(crate) const MAX_CONTROL_FRAME_PAYLOAD_LEN: usize = 125;
pub(crate) const MAX_HDR_LEN_U8: u8 = 14;
pub(crate) const MAX_HDR_LEN_USIZE: usize = 14;
pub(crate) const MIN_HEADER_LEN_USIZE: usize = 2;
pub(crate) const DECOMPRESSION_SUFFIX: &[u8; 4] = &[0, 0, 255, 255];

/// Always masks the payload before sending.
pub type WebSocketClient<NC, PB, RNG, S> = WebSocket<NC, PB, RNG, S, true>;
/// [WebSocketClient] with a mutable reference of [PartitionedBuffer].
pub type WebSocketClientMut<'pb, NC, RNG, S> =
  WebSocketClient<NC, &'pb mut PartitionedBuffer, RNG, S>;
/// [WebSocketClient] with an owned [PartitionedBuffer].
pub type WebSocketClientOwned<NC, RNG, S> = WebSocketClient<NC, PartitionedBuffer, RNG, S>;
/// Always unmasks the payload after receiving.
pub type WebSocketServer<NC, PB, RNG, S> = WebSocket<NC, PB, RNG, S, false>;
/// [WebSocketServer] with a mutable reference of [PartitionedBuffer].
pub type WebSocketServerMut<'pb, NC, RNG, S> =
  WebSocketServer<NC, &'pb mut PartitionedBuffer, RNG, S>;
/// [WebSocketServer] with an owned [PartitionedBuffer].
pub type WebSocketServerOwned<NC, RNG, S> = WebSocketServer<NC, PartitionedBuffer, RNG, S>;

type ReadContinuationFramesCbs<B> = (
  fn(&[u8]) -> crate::Result<Option<IncompleteUtf8Char>>,
  fn(&[u8], &mut Option<IncompleteUtf8Char>) -> crate::Result<()>,
  fn(
    &mut Buffer,
    &mut FrameBuffer<B>,
    &mut PartitionedBuffer,
    &ReadFrameInfo,
    usize,
  ) -> crate::Result<(bool, usize)>,
);

/// WebSocket protocol implementation over an asynchronous stream.
#[derive(Debug)]
pub struct WebSocket<NC, PB, RNG, S, const IS_CLIENT: bool> {
  auto_close: bool,
  auto_pong: bool,
  decompression_buffer: Buffer,
  is_stream_closed: bool,
  max_payload_len: usize,
  nc: NC,
  pb: PB,
  rng: RNG,
  stream: S,
}

impl<NC, PB, RNG, S, const IS_CLIENT: bool> WebSocket<NC, PB, RNG, S, IS_CLIENT> {
  /// Sets whether to automatically close the connection when a close frame is received. Defaults
  /// to `true`.
  #[inline]
  pub fn set_auto_close(&mut self, auto_close: bool) {
    self.auto_close = auto_close;
  }

  /// Sets whether to automatically send a pong frame when a ping frame is received. Defaults
  /// to `true`.
  #[inline]
  pub fn set_auto_pong(&mut self, auto_pong: bool) {
    self.auto_pong = auto_pong;
  }

  /// Sets whether to automatically close the connection when a received frame payload length
  /// exceeds `max_payload_len`. Defaults to `64 * 1024 * 1024` bytes (64 MiB).
  #[inline]
  pub fn set_max_payload_len(&mut self, max_payload_len: usize) {
    self.max_payload_len = max_payload_len;
  }
}

impl<NC, PB, RNG, S, const IS_CLIENT: bool> WebSocket<NC, PB, RNG, S, IS_CLIENT>
where
  NC: NegotiatedCompression,
  PB: BorrowMut<PartitionedBuffer>,
  RNG: Rng,
  S: Stream,
{
  /// Creates a new instance from a stream that supposedly has already completed the WebSocket
  /// handshake.
  #[inline]
  pub fn new(nc: NC, mut pb: PB, rng: RNG, stream: S) -> Self {
    pb.borrow_mut().clear_if_following_is_empty();
    pb.borrow_mut().expand_following(MAX_HDR_LEN_USIZE);
    Self {
      auto_close: true,
      auto_pong: true,
      decompression_buffer: Buffer::default(),
      is_stream_closed: false,
      max_payload_len: MAX_PAYLOAD_LEN,
      nc,
      pb,
      rng,
      stream,
    }
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
    B: AsMut<[u8]> + AsMut<Vec<u8>> + AsRef<[u8]>,
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
            |local_db, _, local_pb, rfi, local_tfl| {
              Ok((true, Self::copy_from_compressed_pb_to_db(local_db, local_tfl, local_pb, rfi)?))
            },
          ),
        )
        .await?;
      let payload_len = Self::copy_from_compressed_db_to_fb(
        &mut self.decompression_buffer,
        fb,
        &mut self.nc,
        payload_start_idx,
      )?;
      let payload = fb
        .buffer()
        .as_ref()
        .get(payload_start_idx..payload_start_idx.wrapping_add(payload_len))
        .unwrap_or_default();
      if matches!(first_rfi.op_code, OpCode::Text) && from_utf8_opt(payload).is_none() {
        return Err(crate::Error::InvalidUTF8);
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
              Ok(match from_utf8_ext_rslt(curr_payload) {
                Err(ExtUtf8Error::Incomplete { incomplete_ending_char, .. }) => {
                  Some(incomplete_ending_char)
                }
                Err(ExtUtf8Error::Invalid { .. }) => {
                  return Err(crate::Error::InvalidUTF8);
                }
                Ok(_) => None,
              })
            },
            if matches!(first_rfi.op_code, OpCode::Binary) {
              |_, _| Ok(())
            } else {
              Self::manage_continuation_text
            },
            |_, local_fb, local_pb, rfi, local_tfl| {
              Ok((
                false,
                Self::copy_from_uncompressed_pb_to_fb(local_fb, local_tfl, local_pb, rfi)?,
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
    B: AsMut<[u8]> + AsRef<[u8]>,
    FB: BorrowMut<FrameBuffer<B>>,
  {
    Self::do_write_frame(
      frame,
      &mut self.is_stream_closed,
      &mut self.nc,
      self.pb.borrow_mut(),
      &mut self.rng,
      &mut self.stream,
    )
    .await?;
    Ok(())
  }

  fn begin_fb_bytes_mut<B>(fb: &mut FrameBuffer<B>, payload_start_idx: usize) -> &mut [u8]
  where
    B: AsMut<[u8]> + AsMut<Vec<u8>>,
  {
    AsMut::<[u8]>::as_mut(fb.buffer_mut()).get_mut(payload_start_idx..).unwrap_or_default()
  }

  fn compress_frame<'pb, B, FB>(
    frame: &Frame<FB, IS_CLIENT>,
    nc: &mut NC,
    pb: &'pb mut PartitionedBuffer,
  ) -> crate::Result<FrameMut<'pb, IS_CLIENT>>
  where
    B: AsMut<[u8]> + AsRef<[u8]>,
    FB: BorrowMut<FrameBuffer<B>>,
  {
    fn expand_pb<'pb, B>(
      len_with_header: usize,
      local_fb: &FrameBuffer<B>,
      local_pb: &'pb mut PartitionedBuffer,
      written: usize,
    ) -> &'pb mut [u8]
    where
      B: AsRef<[u8]>,
    {
      let start = len_with_header.wrapping_add(written);
      local_pb.expand_following(start.wrapping_add(local_fb.frame().len()).wrapping_add(128));
      local_pb.following_trail_mut().get_mut(start..).unwrap_or_default()
    }

    let fb = frame.fb().borrow();
    let len = pb.following_trail_mut().len();
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
      pb.following_trail_mut()
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
    db: &mut Buffer,
    fb: &mut FrameBuffer<B>,
    nc: &mut NC,
    payload_start_idx: usize,
  ) -> crate::Result<usize>
  where
    B: AsMut<[u8]> + AsMut<Vec<u8>> + AsRef<[u8]>,
  {
    db.push_bytes(DECOMPRESSION_SUFFIX);
    let mut buffer_len = payload_start_idx
      .checked_add(db.len())
      .map(|element| element.max(fb.buffer().as_ref().len()));
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
    db: &mut Buffer,
    payload_start_idx: usize,
    pb: &mut PartitionedBuffer,
    rfi: &ReadFrameInfo,
  ) -> crate::Result<usize> {
    Self::copy_from_pb(db, pb, rfi, |local_pb, local_db| {
      let n = payload_start_idx.saturating_add(rfi.payload_len);
      local_db.set_idx_through_expansion(n);
      local_db
        .get_mut(payload_start_idx..n)
        .unwrap_or_default()
        .copy_from_slice(local_pb.current().get(rfi.header_end_idx..).unwrap_or_default());
      Ok(())
    })?;
    Ok(rfi.payload_len)
  }

  // Final compressed single frame
  fn copy_from_compressed_pb_to_fb<B>(
    fb: &mut FrameBuffer<B>,
    nc: &mut NC,
    payload_start_idx: usize,
    pb: &mut PartitionedBuffer,
    rfi: &ReadFrameInfo,
  ) -> crate::Result<usize>
  where
    B: AsMut<[u8]> + AsMut<Vec<u8>> + AsRef<[u8]>,
  {
    let mut buffer_len = payload_start_idx
      .checked_add(rfi.payload_len)
      .map(|element| element.max(fb.buffer().as_ref().len()));
    let payload_len = Self::copy_from_pb(fb, pb, rfi, |local_pb, local_fb| {
      local_pb.expand_buffer(local_pb._buffer().len().wrapping_add(4));
      let curr_end_idx = local_pb.current().len();
      let curr_end_idx_4p = curr_end_idx.wrapping_add(4);
      let has_following = local_pb.has_following();
      let range = rfi.header_end_idx..curr_end_idx_4p;
      let input = local_pb.current_trail_mut().get_mut(range).unwrap_or_default();
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
    pb: &mut PartitionedBuffer,
    rfi: &ReadFrameInfo,
    cb: impl FnOnce(&mut PartitionedBuffer, &mut O) -> crate::Result<T>,
  ) -> crate::Result<T> {
    debug!(
      "{:<5} - {:<5} - {:<25}: {:?}, {:?}",
      <&str>::from(Role::from_is_client(IS_CLIENT)),
      "Read",
      "Masked",
      crate::misc::_truncated_slice(pb.current(), 0..32),
      rfi.op_code
    );

    if !IS_CLIENT {
      unmask(
        pb.current_mut().get_mut(rfi.header_end_idx..).unwrap_or_default(),
        rfi.mask.ok_or(WebSocketError::MissingFrameMask)?,
      );
    }

    debug!(
      "{:<5} - {:<5} - {:<25}: {:?}, {:?}",
      <&str>::from(Role::from_is_client(IS_CLIENT)),
      "Read",
      "Unmasked",
      crate::misc::_truncated_slice(pb.current(), 0..32),
      rfi.op_code
    );

    let rslt = cb(pb, output)?;
    pb.borrow_mut().clear_if_following_is_empty();

    Ok(rslt)
  }

  // Final uncompressed single frame as well as intermediate uncompressed continuation frame
  fn copy_from_uncompressed_pb_to_fb<B>(
    fb: &mut FrameBuffer<B>,
    payload_start_idx: usize,
    pb: &mut PartitionedBuffer,
    rfi: &ReadFrameInfo,
  ) -> crate::Result<usize>
  where
    B: AsMut<[u8]> + AsMut<Vec<u8>>,
  {
    Self::copy_from_pb(fb, pb, rfi, |local_pb, local_fb| {
      let n = payload_start_idx.saturating_add(rfi.payload_len);
      local_fb.expand_buffer(n);
      AsMut::<[u8]>::as_mut(local_fb.buffer_mut())
        .get_mut(payload_start_idx..n)
        .unwrap_or_default()
        .copy_from_slice(local_pb.current().get(rfi.header_end_idx..).unwrap_or_default());
      Ok(())
    })?;
    Ok(rfi.payload_len)
  }

  fn curr_payload_bytes<'bytes, B>(
    db: &'bytes Buffer,
    fb: &'bytes FrameBuffer<B>,
    range: Range<usize>,
    should_use_db: bool,
  ) -> &'bytes [u8]
  where
    B: AsMut<[u8]> + AsMut<Vec<u8>> + AsRef<[u8]>,
  {
    if should_use_db {
      db.get(range).unwrap_or_default()
    } else {
      fb.buffer().as_ref().get(range).unwrap_or_default()
    }
  }

  async fn do_write_frame<B, FB>(
    frame: &mut Frame<FB, IS_CLIENT>,
    is_stream_closed: &mut bool,
    nc: &mut NC,
    pb: &mut PartitionedBuffer,
    rng: &mut RNG,
    stream: &mut S,
  ) -> crate::Result<()>
  where
    B: AsMut<[u8]> + AsRef<[u8]>,
    FB: BorrowMut<FrameBuffer<B>>,
  {
    let mut should_compress = false;
    if frame.op_code() == OpCode::Close {
      *is_stream_closed = true;
    }
    if !frame.op_code().is_control() {
      if let Some(first) = frame.fb_mut().borrow_mut().header_mut().first_mut() {
        should_compress = nc.rsv1() != 0;
        *first |= nc.rsv1();
      }
    }
    if !should_compress || frame.op_code().is_control() {
      debug!(
        "{:<5} - {:<5} - {:<25}: {:?}, {:?}",
        <&str>::from(Role::from_is_client(IS_CLIENT)),
        "Write",
        "Unmasked",
        crate::misc::_truncated_slice(frame.fb().borrow().frame(), 0..32),
        frame.op_code()
      );
      Self::mask_frame(frame, rng);
      debug!(
        "{:<5} - {:<5} - {:<25}: {:?}, {:?}",
        <&str>::from(Role::from_is_client(IS_CLIENT)),
        "Write",
        "Masked",
        crate::misc::_truncated_slice(frame.fb().borrow().frame(), 0..32),
        frame.op_code()
      );
      stream.write_all(frame.fb().borrow().frame()).await?;
    } else {
      debug!(
        "{:<5} - {:<5} - {:<25}: {:?}, {:?}",
        <&str>::from(Role::from_is_client(IS_CLIENT)),
        "Write",
        "Uncompressed, Unmasked",
        crate::misc::_truncated_slice(frame.fb().borrow().frame(), 0..32),
        frame.op_code()
      );
      let mut compressed_frame = Self::compress_frame(frame, nc, pb)?;
      debug!(
        "{:<5} - {:<5} - {:<25}: {:?}, {:?}",
        <&str>::from(Role::from_is_client(IS_CLIENT)),
        "Write",
        "Compressed, Unmasked",
        crate::misc::_truncated_slice(compressed_frame.fb().frame(), 0..32),
        frame.op_code()
      );
      Self::mask_frame(&mut compressed_frame, rng);
      debug!(
        "{:<5} - {:<5} - {:<25}: {:?}, {:?}",
        <&str>::from(Role::from_is_client(IS_CLIENT)),
        "Write",
        "Compressed, Masked",
        crate::misc::_truncated_slice(compressed_frame.fb().frame(), 0..32),
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
  ) -> &'fb mut [u8]
  where
    B: AsMut<[u8]> + AsMut<Vec<u8>>,
  {
    *buffer_len = buffer_len.and_then(|el| el.checked_mul(15)?.checked_div(10));
    fb.expand_buffer(buffer_len.unwrap_or(usize::MAX));
    let start = payload_start_idx.wrapping_add(written);
    Self::begin_fb_bytes_mut(fb, start)
  }

  async fn fetch_frame_from_stream(&mut self) -> crate::Result<ReadFrameInfo> {
    let mut read = self.pb.borrow_mut().following_len();
    let rfi = Self::fetch_header_from_stream(
      self.max_payload_len,
      &self.nc,
      self.pb.borrow_mut(),
      &mut read,
      &mut self.stream,
    )
    .await?;
    if self.is_stream_closed && rfi.op_code != OpCode::Close {
      return Err(WebSocketError::ConnectionClosed.into());
    }
    Self::fetch_payload_from_stream(self.pb.borrow_mut(), &mut read, &rfi, &mut self.stream)
      .await?;
    Ok(rfi)
  }

  async fn fetch_header_from_stream(
    max_payload_len: usize,
    nc: &NC,
    pb: &mut PartitionedBuffer,
    read: &mut usize,
    stream: &mut S,
  ) -> crate::Result<ReadFrameInfo> {
    let buffer = pb.following_trail_mut();

    let first_two = Self::read_until::<2>(buffer, read, 0, stream).await?;

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
      126 => (4, u16::from_be_bytes(Self::read_until::<2>(buffer, read, 2, stream).await?).into()),
      127 => {
        let payload_len = Self::read_until::<8>(buffer, read, 2, stream).await?;
        (10, u64::from_be_bytes(payload_len).try_into()?)
      }
      _ => (2, length_code.into()),
    };

    let mut mask = None;
    if !IS_CLIENT {
      mask = Some(Self::read_until::<4>(buffer, read, header_len, stream).await?);
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
    pb: &mut PartitionedBuffer,
    read: &mut usize,
    rfi: &ReadFrameInfo,
    stream: &mut S,
  ) -> crate::Result<()> {
    let mut is_payload_filled = false;
    pb.expand_following(rfi.frame_len);
    for _ in 0..rfi.frame_len {
      if *read >= rfi.frame_len {
        is_payload_filled = true;
        break;
      }
      *read = read.wrapping_add(
        stream.read(pb.following_trail_mut().get_mut(*read..).unwrap_or_default()).await?,
      );
    }
    if !is_payload_filled {
      return Err(crate::Error::UnexpectedBufferState);
    }
    pb.set_indices(pb.current_end_idx(), rfi.frame_len, read.wrapping_sub(rfi.frame_len))?;
    Ok(())
  }

  // If this method returns `false`, then a `ping` frame was received and the caller should fetch
  // more external data in order to get the desired frame.
  async fn manage_auto_reply(
    (auto_close, auto_pong, is_stream_closed): (bool, bool, &mut bool),
    curr_payload: &[u8],
    nc: &mut NC,
    op_code: OpCode,
    pb: &mut PB,
    rng: &mut RNG,
    stream: &mut S,
  ) -> crate::Result<bool> {
    match op_code {
      OpCode::Close if auto_close && !*is_stream_closed => {
        match curr_payload {
          [] => {}
          [_] => return Err(WebSocketError::InvalidCloseFrame.into()),
          [a, b, rest @ ..] => {
            if from_utf8_opt(rest).is_none() {
              return Err(crate::Error::InvalidUTF8);
            };
            let is_not_allowed = !CloseCode::from(u16::from_be_bytes([*a, *b])).is_allowed();
            if is_not_allowed || rest.len() > MAX_CONTROL_FRAME_PAYLOAD_LEN - 2 {
              Self::write_control_frame(
                &mut FrameControlArray::close_from_params(
                  CloseCode::Protocol,
                  <_>::default(),
                  rest,
                )?,
                is_stream_closed,
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
          &mut FrameControlArray::new_fin(<_>::default(), OpCode::Close, curr_payload)?,
          is_stream_closed,
          nc,
          pb,
          rng,
          stream,
        )
        .await?;
        Ok(true)
      }
      OpCode::Ping if auto_pong => {
        Self::write_control_frame(
          &mut FrameControlArray::new_fin(<_>::default(), OpCode::Pong, curr_payload)?,
          is_stream_closed,
          nc,
          pb,
          rng,
          stream,
        )
        .await?;
        Ok(false)
      }
      OpCode::Continuation
      | OpCode::Binary
      | OpCode::Close
      | OpCode::Ping
      | OpCode::Pong
      | OpCode::Text => Ok(true),
    }
  }

  fn manage_continuation_text(
    curr_payload: &[u8],
    iuc: &mut Option<IncompleteUtf8Char>,
  ) -> crate::Result<()> {
    let tail = if let Some(mut incomplete) = iuc.take() {
      let (rslt, remaining) = incomplete.complete(curr_payload);
      match rslt {
        Err(CompleteErr::HasInvalidBytes) => {
          return Err(crate::Error::InvalidUTF8);
        }
        Err(CompleteErr::InsufficientInput) => {
          let _ = iuc.replace(incomplete);
          &[]
        }
        Ok(_) => remaining,
      }
    } else {
      curr_payload
    };
    match from_utf8_ext_rslt(tail) {
      Err(ExtUtf8Error::Incomplete { incomplete_ending_char, .. }) => {
        *iuc = Some(incomplete_ending_char);
      }
      Err(ExtUtf8Error::Invalid { .. }) => {
        return Err(crate::Error::InvalidUTF8);
      }
      Ok(_) => {}
    }
    Ok(())
  }

  fn mask_frame<B, FB>(frame: &mut Frame<FB, IS_CLIENT>, rng: &mut RNG)
  where
    B: AsMut<[u8]> + AsRef<[u8]>,
    FB: BorrowMut<FrameBuffer<B>>,
  {
    if IS_CLIENT {
      if let [_, second_byte, .., a, b, c, d] = frame.fb_mut().borrow_mut().header_mut() {
        if !has_masked_frame(*second_byte) {
          *second_byte |= 0b1000_0000;
          let mask = rng.u8_4();
          *a = mask[0];
          *b = mask[1];
          *c = mask[2];
          *d = mask[3];
          unmask(frame.fb_mut().borrow_mut().payload_mut(), mask);
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
    B: AsMut<[u8]> + AsMut<Vec<u8>> + AsRef<[u8]>,
  {
    let mut iuc = {
      let (should_use_db, payload_len) = copy_cb(
        &mut self.decompression_buffer,
        fb,
        self.pb.borrow_mut(),
        first_rfi,
        *total_frame_len,
      )?;
      *total_frame_len = total_frame_len.wrapping_add(payload_len);
      match first_rfi.op_code {
        OpCode::Binary => None,
        OpCode::Text => first_text_cb(Self::curr_payload_bytes(
          &self.decompression_buffer,
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
        let (should_use_db, payload_len) = copy_cb(
          &mut self.decompression_buffer,
          fb,
          self.pb.borrow_mut(),
          &rfi,
          *total_frame_len,
        )?;
        *total_frame_len = total_frame_len.wrapping_add(payload_len);
        let curr_payload = Self::curr_payload_bytes(
          &self.decompression_buffer,
          fb,
          prev..*total_frame_len,
          should_use_db,
        );
        if Self::manage_auto_reply(
          (self.auto_close, self.auto_pong, &mut self.is_stream_closed),
          curr_payload,
          &mut self.nc,
          rfi.op_code,
          &mut self.pb,
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

  #[inline]
  async fn read_first_frame<'fb, B>(
    &mut self,
    fb: &'fb mut FrameBuffer<B>,
    header_buffer_len: u8,
    payload_start_idx: usize,
  ) -> crate::Result<Option<ReadFrameInfo>>
  where
    B: AsMut<[u8]> + AsMut<Vec<u8>> + AsRef<[u8]>,
  {
    let first_rfi = 'auto_reply: loop {
      let rfi = self.fetch_frame_from_stream().await?;
      if !rfi.fin {
        break 'auto_reply rfi;
      }
      let pb = self.pb.borrow_mut();
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
      if Self::manage_auto_reply(
        (self.auto_close, self.auto_pong, &mut self.is_stream_closed),
        fb.payload(),
        &mut self.nc,
        rfi.op_code,
        &mut self.pb,
        &mut self.rng,
        &mut self.stream,
      )
      .await?
      {
        match rfi.op_code {
          OpCode::Continuation => {
            return Err(WebSocketError::UnexpectedMessageFrame.into());
          }
          OpCode::Text => {
            if from_utf8_opt(fb.payload()).is_none() {
              return Err(crate::Error::InvalidUTF8);
            }
          }
          OpCode::Binary | OpCode::Close | OpCode::Ping | OpCode::Pong => {}
        }
        return Ok(None);
      }
    };
    Ok(Some(first_rfi))
  }

  async fn read_until<const LEN: usize>(
    buffer: &mut [u8],
    read: &mut usize,
    start: usize,
    stream: &mut S,
  ) -> crate::Result<[u8; LEN]>
  where
    [u8; LEN]: Default,
  {
    let until = start.wrapping_add(LEN);
    while *read < until {
      let actual_buffer = buffer.get_mut(*read..).unwrap_or_default();
      let local_read = stream.read(actual_buffer).await?;
      if local_read == 0 {
        return Err(crate::Error::UnexpectedEOF);
      }
      *read = read.wrapping_add(local_read);
    }
    Ok(buffer.get(start..until).and_then(|el| el.try_into().ok()).unwrap_or_default())
  }

  async fn write_control_frame(
    frame: &mut FrameControlArray<IS_CLIENT>,
    is_stream_closed: &mut bool,
    nc: &mut NC,
    pb: &mut PB,
    rng: &mut RNG,
    stream: &mut S,
  ) -> crate::Result<()> {
    Self::do_write_frame(frame, is_stream_closed, nc, pb.borrow_mut(), rng, stream).await?;
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
