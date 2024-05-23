use crate::{
  misc::{Lease, LeaseMut, SingleTypeStorage},
  web_socket::{
    close_code::CloseCode,
    frame_buffer::{
      FrameBufferControlArray, FrameBufferControlArrayMut, FrameBufferMut, FrameBufferVecMut,
    },
    misc::{define_fb_from_header_params, op_code, Expand},
    FrameBuffer, FrameBufferVec, OpCode, MAX_CONTROL_FRAME_PAYLOAD_LEN, MAX_HDR_LEN_USIZE,
    MIN_HEADER_LEN_USIZE,
  },
};
use core::str;

/// Composed by a [FrameBufferControlArray].
pub type FrameControlArray<const IS_CLIENT: bool> = Frame<FrameBufferControlArray, IS_CLIENT>;
/// Composed by a [FrameBufferControlArrayMut].
pub type FrameControlArrayMut<'bytes, const IS_CLIENT: bool> =
  Frame<FrameBufferControlArrayMut<'bytes>, IS_CLIENT>;
/// Composed by a [FrameBufferMut].
pub type FrameMut<'bytes, const IS_CLIENT: bool> = Frame<FrameBufferMut<'bytes>, IS_CLIENT>;
/// Composed by a [FrameBufferVec].
pub type FrameVec<const IS_CLIENT: bool> = Frame<FrameBufferVec, IS_CLIENT>;
/// Composed by an mutable [FrameBufferVecMut] reference.
pub type FrameVecMut<'bytes, const IS_CLIENT: bool> = Frame<FrameBufferVecMut<'bytes>, IS_CLIENT>;

/// Composed by an mutable [FrameBufferControlArray] reference.
pub type FrameMutControlArray<'fb, const IS_CLIENT: bool> =
  Frame<&'fb mut FrameBufferControlArray, IS_CLIENT>;
/// Composed by an mutable [FrameBufferControlArrayMut] reference.
pub type FrameMutControlArrayMut<'fb, const IS_CLIENT: bool> =
  Frame<&'fb mut FrameBufferControlArray, IS_CLIENT>;
/// Composed by an mutable [FrameBufferMut] reference.
pub type FrameMutMut<'bytes, 'fb, const IS_CLIENT: bool> =
  Frame<&'fb mut FrameBufferMut<'bytes>, IS_CLIENT>;
/// Composed by an mutable [FrameBufferVec] reference.
pub type FrameMutVec<'fb, const IS_CLIENT: bool> = Frame<&'fb mut FrameBufferVec, IS_CLIENT>;
/// Composed by an mutable [FrameBufferVecMut] reference.
pub type FrameMutVecMut<'bytes, 'fb, const IS_CLIENT: bool> =
  Frame<&'fb mut FrameBufferVecMut<'bytes>, IS_CLIENT>;

/// Represents a WebSocket frame
#[derive(Debug)]
pub struct Frame<FB, const IS_CLIENT: bool> {
  fb: FB,
  fin: bool,
  op_code: OpCode,
}

impl<FB, const IS_CLIENT: bool> Frame<FB, IS_CLIENT> {
  /// Contains the raw bytes that compose this frame.
  #[inline]
  pub fn fb(&self) -> &FB {
    &self.fb
  }

  pub(crate) fn fb_mut(&mut self) -> &mut FB {
    &mut self.fb
  }

  /// Indicates if this is the final frame in a message.
  #[inline]
  pub fn fin(&self) -> bool {
    self.fin
  }

  /// See [OpCode].
  #[inline]
  pub fn op_code(&self) -> OpCode {
    self.op_code
  }
}

impl<B, FB, const IS_CLIENT: bool> Frame<FB, IS_CLIENT>
where
  B: Lease<[u8]>,
  FB: Lease<FrameBuffer<B>> + SingleTypeStorage<Item = B>,
{
  /// Creates a new instance based on the contained bytes of `fb`.
  #[inline]
  pub fn from_fb(fb: FB) -> crate::Result<Self> {
    let header = fb.lease().header();
    let len = header.len();
    let has_valid_header = (MIN_HEADER_LEN_USIZE..=MAX_HDR_LEN_USIZE).contains(&len);
    let (true, Some(first_header_byte)) = (has_valid_header, header.first().copied()) else {
      return Err(crate::Error::WS_InvalidFrameHeaderBounds);
    };
    Ok(Self { fb, fin: first_header_byte & 0b1000_0000 != 0, op_code: op_code(first_header_byte)? })
  }

  /// If the frame is of type [OpCode::Text], returns its payload interpreted as a string.
  #[inline]
  pub fn text_payload<'this>(&'this self) -> Option<&'this str>
  where
    B: 'this,
  {
    self.op_code.is_text().then(|| {
      // SAFETY: UTF-8 data is always verified when read from a stream.
      unsafe { str::from_utf8_unchecked(self.fb.lease().payload()) }
    })
  }
}

impl<B, FB, const IS_CLIENT: bool> Frame<FB, IS_CLIENT>
where
  B: Expand + LeaseMut<[u8]>,
  FB: LeaseMut<FrameBuffer<B>> + SingleTypeStorage<Item = B>,
{
  /// Creates based on the individual parameters that compose a close frame.
  ///
  /// `reason` is capped based on the maximum allowed size of a control frame minus 2.
  #[inline]
  pub fn close_from_params(code: CloseCode, fb: FB, reason: &[u8]) -> crate::Result<Self> {
    let reason_len = reason.len().min(MAX_CONTROL_FRAME_PAYLOAD_LEN - 2);
    let payload_len = reason_len.wrapping_add(2);
    Self::build_frame(fb, true, OpCode::Close, payload_len, |local_fb| {
      let payload = local_fb.lease_mut().payload_mut();
      payload.get_mut(..2).unwrap_or_default().copy_from_slice(&u16::from(code).to_be_bytes());
      payload
        .get_mut(2..)
        .unwrap_or_default()
        .copy_from_slice(reason.get(..reason_len).unwrap_or_default());
      Ok(())
    })
  }

  /// Creates a new instance that is considered final.
  #[inline]
  pub fn new_fin(fb: FB, op_code: OpCode, payload: &[u8]) -> crate::Result<Self> {
    Self::new(fb, true, op_code, payload)
  }

  /// Creates a new instance that is meant to be a continuation of previous frames.
  #[inline]
  pub fn new_unfin(fb: FB, op_code: OpCode, payload: &[u8]) -> crate::Result<Self> {
    Self::new(fb, false, op_code, payload)
  }

  fn build_frame(
    mut fb: FB,
    fin: bool,
    op_code: OpCode,
    payload_len: usize,
    cb: impl FnOnce(&mut FB) -> crate::Result<()>,
  ) -> crate::Result<Self> {
    fb.lease_mut().clear();
    fb.lease_mut().buffer_mut().expand(MAX_HDR_LEN_USIZE.saturating_add(payload_len));
    define_fb_from_header_params::<_, IS_CLIENT>(
      fb.lease_mut(),
      fin,
      None,
      op_code,
      payload_len,
      0,
    )?;
    cb(&mut fb)?;
    Ok(Self { fb, fin, op_code })
  }

  fn new(fb: FB, fin: bool, op_code: OpCode, payload: &[u8]) -> crate::Result<Self> {
    let payload_len = if op_code.is_control() {
      payload.len().min(MAX_CONTROL_FRAME_PAYLOAD_LEN)
    } else {
      payload.len()
    };
    Self::build_frame(fb, fin, op_code, payload_len, |local_fb| {
      local_fb
        .lease_mut()
        .payload_mut()
        .copy_from_slice(payload.get(..payload_len).unwrap_or_default());
      Ok(())
    })
  }
}
