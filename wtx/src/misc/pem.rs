use crate::{
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  collection::{ArrayStringU8, ArrayVectorU8, ExpansionTy, TryExtend, Vector},
  misc::{Lease, LeaseMut, bytes_split_once1},
};
use base64::{Engine, prelude::BASE64_STANDARD};
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
  fn decode(dw: &mut GenericDecodeWrapper<'_, T>) -> crate::Result<Self> {
    let mut this = Self { data: ArrayVectorU8::new() };
    let GenericDecodeWrapper { bytes, decode_aux } = dw;
    let idx = decode_aux.lease().len();
    decode_aux.try_extend((0, bytes.len()))?;
    let output = decode_aux.lease_mut().get_mut(idx..).unwrap_or_default();
    let mut output_idx = 0;
    while let Some((first_line, rest)) = bytes_split_once1(bytes, b'\n') {
      *bytes = rest;
      let actual_first_line = strip_cr(first_line);
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
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, &mut Vector<u8>>) -> crate::Result<()> {
    let buffer_base64 = &mut ew.encode_aux;
    let buffer_out = &mut ew.buffer;
    let idx = buffer_base64.len();
    for (label_string, block) in &self.data {
      let label = label_string.as_bytes();
      let decoded = block.lease();
      let additional = base64::encoded_len(decoded.len(), true).unwrap_or_default();
      buffer_base64.truncate(idx);
      buffer_base64.expand(ExpansionTy::Additional(additional), 0)?;
      let output = buffer_base64.get_mut(idx..).unwrap_or_default();
      let len = BASE64_STANDARD.encode_slice(decoded, output)?;
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
    return Err(crate::Error::InvalidPem);
  };
  while let Some((line, rest)) = bytes_split_once1(bytes, b'\n') {
    *bytes = rest;
    let actual_line = strip_cr(line);
    if let [
      b'-', b'-', b'-', b'-', b'-', b'E', b'N', b'D', b' ',
      label_end @ ..,
      b'-', b'-', b'-', b'-', b'-'
    ] = actual_line
    {
      if label_begin != label_end {
        return Err(crate::Error::InvalidPem);
      }
      break;
    } else {
      let local_output = output.get_mut(*output_idx..).unwrap_or_default();
      let len = BASE64_STANDARD.decode_slice(actual_line, local_output)?;
      *output_idx = output_idx.wrapping_add(len);
    }
  }
  label_begin.try_into()
}

#[inline]
fn strip_cr(bytes: &[u8]) -> &[u8] {
  if let [rest @ .., b'\r'] = bytes { rest } else { bytes }
}
