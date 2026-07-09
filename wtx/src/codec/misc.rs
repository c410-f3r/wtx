use crate::{
  codec::{
    Base64Alphabet, HexEncMode, base64_decode, base64_decoded_len_ub, base64_encode,
    base64_encoded_len, hex_decode, hex_encode,
  },
  collections::{CapacityUpperBound, LinearStorageLen, Truncate, TryExtend},
  misc::{Lease as _, LeaseMut},
};

/// Decodes Base64 into a buffer
#[inline]
pub fn decode_base64_into_buffer<'buffer, B, L>(
  buffer: &'buffer mut B,
  bytes: &[u8],
) -> crate::Result<&'buffer mut [u8]>
where
  B: CapacityUpperBound + LeaseMut<[u8]> + Truncate<L> + TryExtend<(u8, usize)>,
  L: LinearStorageLen,
{
  let max_decoded_len = base64_decoded_len_ub(bytes.len());
  decode_into_buffer(buffer, max_decoded_len.min(B::CAPACITY_UPPER_BOUND), |slice| {
    Ok(base64_decode(Base64Alphabet::Standard, bytes, slice)?.len())
  })
}

/// Decodes Hex into a buffer
#[inline]
pub fn decode_hex_into_buffer<'buffer, B, L>(
  buffer: &'buffer mut B,
  bytes: &[u8],
) -> crate::Result<&'buffer mut [u8]>
where
  B: CapacityUpperBound + LeaseMut<[u8]> + Truncate<L> + TryExtend<(u8, usize)>,
  L: LinearStorageLen,
{
  let max_decoded_len = bytes.len() / 2;
  decode_into_buffer(buffer, max_decoded_len.min(B::CAPACITY_UPPER_BOUND), |slice| {
    Ok(hex_decode(bytes, slice).map_err(crate::Error::from)?.len())
  })
}

/// Encodes Base64 into a buffer
#[inline]
pub fn encode_base64_into_buffer<'buffer, B, L>(
  alphabet: Base64Alphabet,
  buffer: &'buffer mut B,
  bytes: &[u8],
) -> crate::Result<&'buffer str>
where
  B: CapacityUpperBound + LeaseMut<[u8]> + Truncate<L> + TryExtend<(u8, usize)>,
  L: LinearStorageLen,
{
  let max_encoded_len = base64_encoded_len(bytes.len(), true).unwrap_or_default();
  encode_into_buffer(buffer, max_encoded_len.min(B::CAPACITY_UPPER_BOUND), |slice| {
    Ok(base64_encode(alphabet, bytes, slice)?.len())
  })
}

/// Encodes Hex into a buffer
#[inline]
pub fn encode_hex_into_buffer<'buffer, B, L>(
  hex_mode: Option<HexEncMode>,
  buffer: &'buffer mut B,
  bytes: &[u8],
) -> crate::Result<&'buffer str>
where
  B: CapacityUpperBound + LeaseMut<[u8]> + Truncate<L> + TryExtend<(u8, usize)>,
  L: LinearStorageLen,
{
  let max_encoded_len = bytes.len().checked_mul(2).unwrap_or_default();
  encode_into_buffer(buffer, max_encoded_len.min(B::CAPACITY_UPPER_BOUND), |slice| {
    Ok(hex_encode(bytes, hex_mode, slice).map_err(crate::Error::from)?.len())
  })
}

#[inline]
fn decode_into_buffer<B, L>(
  buffer: &mut B,
  max_decoded_len: usize,
  cb: impl FnOnce(&mut [u8]) -> crate::Result<usize>,
) -> crate::Result<&mut [u8]>
where
  B: LeaseMut<[u8]> + Truncate<L> + TryExtend<(u8, usize)>,
  L: LinearStorageLen,
{
  let prev = buffer.lease().len();
  buffer.try_extend((0, max_decoded_len))?;
  let slice = buffer.lease_mut().get_mut(prev..).unwrap_or_default();
  let len = cb(slice)?;
  buffer.truncate(L::from_usize(prev.wrapping_add(len))?);
  Ok(buffer.lease_mut().get_mut(prev..).unwrap_or_default())
}

#[inline]
fn encode_into_buffer<B, L>(
  buffer: &mut B,
  max_encoded_len: usize,
  cb: impl FnOnce(&mut [u8]) -> crate::Result<usize>,
) -> crate::Result<&str>
where
  B: LeaseMut<[u8]> + Truncate<L> + TryExtend<(u8, usize)>,
  L: LinearStorageLen,
{
  let prev = buffer.lease().len();
  buffer.try_extend((0, max_encoded_len))?;
  let slice = buffer.lease_mut().get_mut(prev..).unwrap_or_default();
  let len = cb(slice)?;
  buffer.truncate(L::from_usize(prev.wrapping_add(len))?);
  // SAFETY: calling functions produce valid UTF-8
  Ok(unsafe { str::from_utf8_unchecked(buffer.lease_mut().get_mut(prev..).unwrap_or_default()) })
}
