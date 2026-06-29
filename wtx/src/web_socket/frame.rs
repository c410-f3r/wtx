use crate::{
  collections::{ArrayVectorCopy, Vector},
  misc::{Lease, from_utf8_basic},
  web_socket::{
    CloseCode, MASK_MASK, MAX_CONTROL_PAYLOAD_LEN, MAX_HEADER_LEN, OpCode, WebSocketError,
    misc::{has_masked_frame, header_from_params},
  },
};
use core::{hint::unreachable_unchecked, str};

/// Composed by an array with the maximum allowed size of a frame control.
pub type FrameControlArray = Frame<ArrayVectorCopy<u8, MAX_CONTROL_PAYLOAD_LEN>>;
/// Composed by a sequence of mutable bytes.
pub type FrameMut<'bytes> = Frame<&'bytes mut [u8]>;
/// Composed by a sequence of immutable bytes.
pub type FrameRef<'bytes> = Frame<&'bytes [u8]>;
/// Composed by an owned vector.
pub type FrameVector = Frame<Vector<u8>>;
/// Composed by a mutable vector reference.
pub type FrameVectorMut<'bytes> = Frame<&'bytes mut Vector<u8>>;
/// Composed by a immutable vector reference.
pub type FrameVectorRef<'bytes> = Frame<&'bytes Vector<u8>>;

/// Unit of generic data used for communication.
#[derive(Debug)]
pub struct Frame<P> {
  fin: bool,
  header: ArrayVectorCopy<u8, MAX_HEADER_LEN>,
  op_code: OpCode,
  payload: P,
}

impl<P> Frame<P> {
  /// Indicates if this is the final frame in a message.
  #[inline]
  pub const fn fin(&self) -> bool {
    self.fin
  }

  /// Header and payload bytes
  #[inline]
  pub fn header_and_payload(&self) -> (&[u8], &P) {
    (self.header(), &self.payload)
  }

  /// See [`OpCode`].
  #[inline]
  pub const fn op_code(&self) -> OpCode {
    self.op_code
  }

  /// Frame's content.
  #[inline]
  pub const fn payload(&self) -> &P {
    &self.payload
  }

  /// Mutable version of [`Self::payload`].
  #[inline]
  pub const fn payload_mut(&mut self) -> &mut P {
    &mut self.payload
  }

  pub(crate) fn header(&self) -> &[u8] {
    &self.header
  }

  pub(crate) fn header_and_payload_mut(&mut self) -> (&mut [u8], &mut P) {
    (&mut self.header, &mut self.payload)
  }

  pub(crate) fn header_first_two_mut(&mut self) -> [&mut u8; 2] {
    let [b0, b1, ..] = self.header.as_slice_mut() else {
      // SAFETY: All constructors have a header of at least 2 bytes
      unsafe { unreachable_unchecked() }
    };
    [b0, b1]
  }

  pub(crate) fn set_mask(&mut self, mask: [u8; 4]) {
    let [_, b1] = self.header_first_two_mut();
    if has_masked_frame(*b1) {
      return;
    }
    *b1 |= MASK_MASK;
    let _rslt = self.header.extend_from_copyable_slice(&mask);
  }
}

impl<P> Frame<P>
where
  P: Lease<[u8]>,
{
  /// Creates a new binary instance that is considered final.
  #[inline]
  pub fn new_fin(op_code: OpCode, payload: P) -> crate::Result<Self> {
    let bytes = payload.lease();
    check_frame(bytes, op_code)?;
    Ok(Self::new(true, op_code, payload, 0))
  }

  /// Unsafe version of [`Self::new_fin`].
  ///
  /// # SAFETY
  ///
  /// * If `op_code` is `Text`, `payload` must be valid UTF-8.
  /// * If `op_code` is `Close`, the payload after the first 2 bytes must be valid UTF-8.
  /// * If `op_code` is a control frame, `payload` must not exceed 125 bytes.
  #[inline]
  pub unsafe fn new_fin_unchecked(op_code: OpCode, payload: P) -> Self {
    Self::new(true, op_code, payload, 0)
  }

  /// Creates a new binary instance that is meant to be a continuation of previous frames.
  #[inline]
  pub fn new_unfin(op_code: OpCode, payload: P) -> crate::Result<Self> {
    let bytes = payload.lease();
    check_frame(bytes, op_code)?;
    Ok(Self::new(false, op_code, payload, 0))
  }

  /// Unsafe version of [`Self::new_unfin`].
  ///
  /// # SAFETY
  ///
  /// * If `op_code` is `Text`, `payload` must be valid UTF-8.
  /// * If `op_code` is `Close`, the payload after the first 2 bytes must be valid UTF-8.
  /// * If `op_code` is a control frame, `payload` must not exceed 125 bytes.
  #[inline]
  pub unsafe fn new_unfin_unchecked(op_code: OpCode, payload: P) -> Self {
    Self::new(false, op_code, payload, 0)
  }

  /// If the frame is of type [`OpCode::Text`], returns its payload interpreted as a string.
  #[inline]
  pub fn text_payload(&self) -> Option<&str> {
    self.op_code.is_text().then(|| {
      // SAFETY:
      // # Instantiating
      //
      // No constructor allows the insertion of non UTF8 data if the [`OpCode`] is Text.
      //
      // # Reading
      //
      // * Single `FIN` frame (No decompression): Whole payload is verified.
      // * Continuation frames (No decompression): Whole payload is verified when concatenated.
      // * Single `FIN` frame (With decompression): Whole payload is verified.
      // * Continuation frames (With decompression): Whole payload is verified when concatenated.
      unsafe { str::from_utf8_unchecked(self.payload.lease()) }
    })
  }

  /// Performs a heap allocation to create a [`FrameVector`] instance.
  #[inline]
  pub fn to_vector(&self) -> crate::Result<FrameVector> {
    Ok(FrameVector {
      fin: self.fin,
      header: self.header,
      op_code: self.op_code,
      payload: Vector::from_copyable_slice(self.payload.lease())?,
    })
  }

  pub(crate) fn new(fin: bool, op_code: OpCode, payload: P, rsv1: u8) -> Self {
    let payload_len = if op_code.is_control() {
      payload.lease().len().min(MAX_CONTROL_PAYLOAD_LEN)
    } else {
      payload.lease().len()
    };
    Self { fin, header: header_from_params(fin, op_code, payload_len, rsv1), op_code, payload }
  }
}

#[inline]
fn check_frame(bytes: &[u8], op_code: OpCode) -> crate::Result<()> {
  if op_code.is_text() {
    let _str = from_utf8_basic(bytes)?;
  } else if op_code.is_close() {
    match bytes {
      [] => {}
      [_] => return Err(WebSocketError::InvalidCloseFrame.into()),
      [b0, b1, rest @ ..] => {
        let _close_code = CloseCode::try_from(u16::from_be_bytes([*b0, *b1]))?;
        let _str = from_utf8_basic(rest)?;
      }
    }
  } else if op_code.is_control() && bytes.len() > MAX_CONTROL_PAYLOAD_LEN {
    return Err(WebSocketError::InvalidControlFrame.into());
  }
  Ok(())
}
