/// Calculates the encoded length of `bytes_len`. Returns `None` is case of an overflow.
#[must_use]
const fn _encoded_len(bytes_len: usize, padding: bool) -> Option<usize> {
  let rem = bytes_len % 3;
  let chunks = bytes_len / 3;
  let Some(len) = chunks.checked_mul(4) else {
    return None;
  };
  if rem > 0 {
    if padding { len.checked_add(4) } else { len.checked_add(rem.wrapping_add(1)) }
  } else {
    Some(len)
  }
}
