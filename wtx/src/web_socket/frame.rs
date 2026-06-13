use crate::{
  collection::{ArrayVectorU8, Vector},
  misc::{Lease, from_utf8_basic},
  web_socket::{
    MASK_MASK, MAX_CONTROL_PAYLOAD_LEN, MAX_HEADER_LEN, OpCode,
    misc::{has_masked_frame, header_from_params},
  },
};
use core::{hint::unreachable_unchecked, str};

/// Composed by an array with the maximum allowed size of a frame control.
pub type FrameControlArray = Frame<ArrayVectorU8<u8, MAX_CONTROL_PAYLOAD_LEN>>;
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
  header: ArrayVectorU8<u8, MAX_HEADER_LEN>,
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
    let [a, b, ..] = self.header.as_slice_mut() else {
      // All constructors have a header of at least 2 bytes
      unsafe { unreachable_unchecked() }
    };
    [a, b]
  }

  pub(crate) fn set_mask(&mut self, mask: [u8; 4]) {
    if has_masked_frame(self.header[1]) {
      return;
    }
    if let Some(first) = self.header.first_mut() {
      *first |= MASK_MASK;
    }
    let _rslt = self.header.push(mask[0]);
    let _rslt = self.header.push(mask[1]);
    let _rslt = self.header.push(mask[2]);
    let _rslt = self.header.push(mask[3]);
  }
}

impl<P> Frame<P>
where
  P: Lease<[u8]>,
{
  /// Creates a new binary instance that is considered final.
  #[inline]
  pub fn new_fin(op_code: OpCode, payload: P) -> crate::Result<Self> {
    if !op_code.is_binary() {
      let _str = from_utf8_basic(payload.lease())?;
    }
    Ok(Self::new(true, op_code, payload, 0))
  }

  /// Unsafe version of [`Self::new_fin`].
  ///
  /// # SAFETY
  ///
  /// You must ensure that `payload` is UTF-8 if `op_code` is not binary.
  #[inline]
  pub unsafe fn new_fin_unchecked(op_code: OpCode, payload: P) -> Self {
    Self::new(true, op_code, payload, 0)
  }

  /// Creates a new binary instance that is meant to be a continuation of previous frames.
  #[inline]
  pub fn new_unfin(op_code: OpCode, payload: P) -> crate::Result<Self> {
    if !op_code.is_binary() {
      let _str = from_utf8_basic(payload.lease())?;
    }
    Ok(Self::new(false, op_code, payload, 0))
  }

  /// Unsafe version of [`Self::new_unfin`].
  ///
  /// # SAFETY
  ///
  /// You must ensure that `payload` is UTF-8 if `op_code` is not binary.
  #[inline]
  pub unsafe fn new_unfin_unchecked(op_code: OpCode, payload: P) -> Self {
    Self::new(true, op_code, payload, 0)
  }

  /// If the frame is of type [`OpCode::Text`], returns its payload interpreted as a string.
  #[inline]
  pub fn text_payload(&self) -> Option<&str> {
    matches!(self.op_code, OpCode::Text | OpCode::Close).then(|| {
      // SAFETY:
      // # Instantiating
      //
      // No constructor allows the insertion of non UTF8 data if the [`OpCode`] is string.
      //
      // # Reading
      //
      // * Single `FIN` frame (No decompression): Whole payload is verified.
      // * Continuation frames (No decompression): Whole payload is verified when concatenated.
      // * Single `FIN` frame (With decompression): Whole payload is verified.
      // * Continuation frames (With decompression): Whole payload is verified when concatenated.
      // * Control frames: Only close frames are verified.
      unsafe { str::from_utf8_unchecked(self.payload.lease()) }
    })
  }

  /// Performs a heap allocation to create a [`FrameVector`] instance.
  #[inline]
  pub fn to_vector(&self) -> crate::Result<FrameVector> {
    Ok(FrameVector {
      fin: self.fin,
      header: self.header.clone(),
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
