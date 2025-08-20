use crate::{
  collection::{ArrayVectorU8, Vector},
  misc::Lease,
  web_socket::{
    MASK_MASK, MAX_CONTROL_PAYLOAD_LEN, MAX_HEADER_LEN, OpCode,
    misc::{fill_header_from_params, has_masked_frame},
  },
};
use core::str;

/// Composed by an array with the maximum allowed size of a frame control.
pub type FrameControlArray<const IS_CLIENT: bool> =
  Frame<ArrayVectorU8<u8, MAX_CONTROL_PAYLOAD_LEN>, IS_CLIENT>;
/// Composed by a sequence of mutable bytes.
pub type FrameMut<'bytes, const IS_CLIENT: bool> = Frame<&'bytes mut [u8], IS_CLIENT>;
/// Composed by a sequence of immutable bytes.
pub type FrameRef<'bytes, const IS_CLIENT: bool> = Frame<&'bytes [u8], IS_CLIENT>;
/// Composed by an owned vector.
pub type FrameVector<const IS_CLIENT: bool> = Frame<Vector<u8>, IS_CLIENT>;
/// Composed by a mutable vector reference.
pub type FrameVectorMut<'bytes, const IS_CLIENT: bool> = Frame<&'bytes mut Vector<u8>, IS_CLIENT>;
/// Composed by a immutable vector reference.
pub type FrameVectorRef<'bytes, const IS_CLIENT: bool> = Frame<&'bytes Vector<u8>, IS_CLIENT>;

/// Unit of generic data used for communication.
#[derive(Debug)]
pub struct Frame<P, const IS_CLIENT: bool> {
  fin: bool,
  header: [u8; MAX_HEADER_LEN],
  header_len: u8,
  op_code: OpCode,
  payload: P,
}

impl<P, const IS_CLIENT: bool> Frame<P, IS_CLIENT> {
  /// Indicates if this is the final frame in a message.
  #[inline]
  pub fn fin(&self) -> bool {
    self.fin
  }

  /// Header and payload bytes
  #[inline]
  pub fn header_and_payload(&self) -> (&[u8], &P) {
    (self.header(), &self.payload)
  }

  /// See [`OpCode`].
  #[inline]
  pub fn op_code(&self) -> OpCode {
    self.op_code
  }

  /// Frame's content.
  #[inline]
  pub fn payload(&self) -> &P {
    &self.payload
  }

  /// Mutable version of [`Self::payload`].
  #[inline]
  pub fn payload_mut(&mut self) -> &mut P {
    &mut self.payload
  }

  pub(crate) fn header(&self) -> &[u8] {
    self.header.get(..self.header_len.into()).unwrap_or_default()
  }

  pub(crate) fn header_and_payload_mut(&mut self) -> (&mut [u8], &mut P) {
    (self.header.get_mut(..self.header_len.into()).unwrap_or_default(), &mut self.payload)
  }

  pub(crate) fn header_first_two_mut(&mut self) -> [&mut u8; 2] {
    let [a, b, ..] = &mut self.header;
    [a, b]
  }

  pub(crate) fn set_mask(&mut self, mask: [u8; 4]) {
    if has_masked_frame(self.header[1]) {
      return;
    }
    self.header_len = self.header_len.wrapping_add(4);
    if let Some([_, a, .., b, c, d, e]) = self.header.get_mut(..self.header_len.into()) {
      *a |= MASK_MASK;
      *b = mask[0];
      *c = mask[1];
      *d = mask[2];
      *e = mask[3];
    }
  }
}

impl<P, const IS_CLIENT: bool> Frame<P, IS_CLIENT>
where
  P: Lease<[u8]>,
{
  /// Creates a new instance that is considered final.
  #[inline]
  pub fn new_fin(op_code: OpCode, payload: P) -> Self {
    Self::new(true, op_code, payload, 0)
  }

  /// Creates a new instance that is meant to be a continuation of previous frames.
  #[inline]
  pub fn new_unfin(op_code: OpCode, payload: P) -> Self {
    Self::new(false, op_code, payload, 0)
  }

  /// If the frame is of type [`OpCode::Text`], returns its payload interpreted as a string.
  #[inline]
  pub fn text_payload(&self) -> Option<&str> {
    matches!(self.op_code, OpCode::Text | OpCode::Close).then(|| {
      // SAFETY:
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
  pub fn to_vector(&self) -> crate::Result<FrameVector<IS_CLIENT>> {
    Ok(FrameVector {
      fin: self.fin,
      header: self.header,
      header_len: self.header_len,
      op_code: self.op_code,
      payload: Vector::from_copyable_slice(self.payload.lease())?,
    })
  }

  pub(crate) fn new(fin: bool, op_code: OpCode, payload: P, rsv1: u8) -> Self {
    let mut header = [0; MAX_HEADER_LEN];
    let payload_len = if op_code.is_control() {
      payload.lease().len().min(MAX_CONTROL_PAYLOAD_LEN)
    } else {
      payload.lease().len()
    };
    let len = fill_header_from_params::<IS_CLIENT>(fin, &mut header, op_code, payload_len, rsv1);
    Self { fin, header, header_len: len, op_code, payload }
  }
}
