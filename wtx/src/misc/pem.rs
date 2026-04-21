use crate::{
  codec::{
    Base64Alphabet, CodecError, Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec,
    base64_decode, base64_decoded_len_ub, base64_encode, base64_encoded_len,
  },
  collection::{ArrayStringU8, ArrayVectorU8, ExpansionTy, TryExtend, Vector},
  misc::{Lease, LeaseMut, bytes_split_once1, strip_new_line},
};
use core::ops::Range;

const MAX_LABEL_LEN: usize = 23;

/// Privacy-enhanced mail (PEM)
///
/// Can decode `CRLF` or `LF` but will always encode with `LF`.
///
/// <https://datatracker.ietf.org/doc/html/rfc7468>
///
/// ```ignore
/// -----BEGIN FOO-----
/// ...Base64 data...
/// ...Base64 data...
/// ...Base64 data...
/// -----END FOO-----
///
/// -----BEGIN BAR-----
/// ...Base64 data...
/// ...Base64 data...
/// ...Base64 data...
/// -----END BAR-----
/// ```
///
/// # Types
///
/// * `B`: Maximum number of blocks
#[derive(Debug, PartialEq)]
pub struct Pem<T, const B: usize> {
  /// Vector of labels and their associated decoded contents
  pub data: ArrayVectorU8<(ArrayStringU8<MAX_LABEL_LEN>, T), B>,
}

impl<T, const B: usize> Decode<'_, GenericCodec<T, ()>> for Pem<Range<usize>, B>
where
  T: LeaseMut<[u8]> + TryExtend<(u8, usize)>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'_, T>) -> crate::Result<Self> {
    let mut this = Self { data: ArrayVectorU8::new() };
    let DecodeWrapper { bytes, decode_aux } = dw;
    let idx = decode_aux.lease().len();
    let additional = base64_decoded_len_ub(bytes.len());
    decode_aux.try_extend((0, additional))?;
    let output = decode_aux.lease_mut().get_mut(idx..).unwrap_or_default();
    let mut output_idx = 0;
    while let Some((first_line, rest)) = bytes_split_once1(bytes, b'\n') {
      *bytes = rest;
      let (_suffixes, actual_first_line) = strip_new_line(first_line);
      if actual_first_line.is_empty() {
        continue;
      }
      let begin = idx.wrapping_add(output_idx);
      let label = parse_block(bytes, actual_first_line, output, &mut output_idx)?;
      let end = idx.wrapping_add(output_idx);
      this.data.push((label, begin..end))?;
    }
    Ok(this)
  }
}

impl<T, const B: usize> Encode<GenericCodec<(), &mut Vector<u8>>> for Pem<T, B>
where
  T: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, &mut Vector<u8>>) -> crate::Result<()> {
    let buffer_base64 = &mut *ew.encode_aux;
    let buffer_out = &mut *ew.buffer;
    let idx = buffer_base64.len();
    for (label_string, block) in &self.data {
      let label = label_string.as_bytes();
      let decoded = block.lease();
      let additional = base64_encoded_len(decoded.len(), true).unwrap_or_default();
      buffer_base64.truncate(idx);
      buffer_base64.expand(ExpansionTy::Additional(additional), 0)?;
      let output = buffer_base64.get_mut(idx..).unwrap_or_default();
      let len = base64_encode(Base64Alphabet::Standard, decoded, output)?.len();
      let encoded = buffer_base64.get(idx..idx.wrapping_add(len)).unwrap_or_default();
      let _ = buffer_out.extend_from_copyable_slices([b"-----BEGIN ", label, b"-----\n"])?;
      let (lines, final_line) = encoded.as_chunks::<64>();
      for line in lines {
        let _ = buffer_out.extend_from_copyable_slices([line.as_slice(), b"\n"])?;
      }
      if !final_line.is_empty() {
        let _ = buffer_out.extend_from_copyable_slices([final_line, b"\n"])?;
      }
      let _ = buffer_out.extend_from_copyable_slices([b"-----END ", label, b"-----\n"])?;
    }
    Ok(())
  }
}

#[inline]
#[rustfmt::skip]
fn parse_block(
  bytes: &mut &[u8],
  first_line: &[u8],
  output: &mut [u8],
  output_idx: &mut usize,
) -> crate::Result<ArrayStringU8<MAX_LABEL_LEN>> {
  let [
    b'-', b'-', b'-', b'-', b'-', b'B', b'E', b'G', b'I', b'N', b' ',
    label_begin @ ..,
    b'-', b'-', b'-', b'-', b'-'
  ] = first_line else {
    return Err(CodecError::InvalidPemLabel.into());
  };
  let mut lines = 0usize;
  while let Some((line, rest)) = bytes_split_once1(bytes, b'\n') {
    *bytes = rest;
    let (_suffixes, actual_line) = strip_new_line(line);
    if let [
      b'-', b'-', b'-', b'-', b'-', b'E', b'N', b'D', b' ',
      label_end @ ..,
      b'-', b'-', b'-', b'-', b'-'
    ] = actual_line
    {
      if label_begin != label_end {
        return Err(CodecError::InvalidPemLabel.into());
      }
      break;
    }
    let local_output = output.get_mut(*output_idx..).unwrap_or_default();
    let len = base64_decode(Base64Alphabet::Standard, actual_line, local_output)?.len();
    *output_idx = output_idx.wrapping_add(len);
    lines = lines.wrapping_add(1);
  }
  if lines <= 2 {
    return Err(CodecError::InvalidPemBlock.into());
  }
  label_begin.try_into()
}
