use crate::misc::Vector;
use core::str;

const ASCII_RANGE_LEN: usize = 128;
const BITS_PER_CHUNK: usize = 32;

/// Characters or bytes in the ASCII range.
#[derive(Clone, Copy, Debug)]
pub struct AsciiSet {
  mask: [u32; ASCII_RANGE_LEN / BITS_PER_CHUNK],
}

impl AsciiSet {
  /// Characters from 0x00 to 0x1F (C0 controls), and 0x7F (DEL).
  ///
  /// <https://url.spec.whatwg.org/#c0-control-percent-encode-set>
  pub const CONTROLS: &AsciiSet = &AsciiSet { mask: [u32::MAX, 0, 0, 2_147_483_648] };

  /// An empty set.
  pub const EMPTY: AsciiSet = AsciiSet { mask: [0; ASCII_RANGE_LEN / BITS_PER_CHUNK] };

  /// Everything but letters or digits.
  pub const NON_ALPHANUMERIC: AsciiSet =
    AsciiSet { mask: [u32::MAX, 4_227_923_967, 4_160_749_569, 4_160_749_569] };

  /// Adds a character into the set.
  #[inline]
  #[must_use]
  pub fn insert(self, byte: u8) -> Self {
    let mut mask = self.mask;
    let byte_usize = usize::from(byte);
    let idx = byte_usize / BITS_PER_CHUNK;
    if let Some(elem) = mask.get_mut(idx) {
      *elem |= 1 << (byte_usize % BITS_PER_CHUNK);
    }
    Self { mask }
  }

  /// Removes a character from the set.
  #[inline]
  #[must_use]
  pub fn remove(self, byte: u8) -> Self {
    let mut mask = self.mask;
    let byte_usize = usize::from(byte);
    let idx = byte_usize / BITS_PER_CHUNK;
    if let Some(elem) = mask.get_mut(idx) {
      *elem &= !(1 << (byte_usize % BITS_PER_CHUNK));
    }
    Self { mask }
  }

  #[inline]
  fn contains(self, byte: u8) -> bool {
    let byte_usize = usize::from(byte);
    let idx = byte_usize / BITS_PER_CHUNK;
    let chunk = self.mask.get(idx).copied().unwrap_or_default();
    let mask = 1 << (byte_usize % BITS_PER_CHUNK);
    (chunk & mask) != 0
  }

  #[inline]
  fn should_percent_encode(self, byte: u8) -> bool {
    !byte.is_ascii() || self.contains(byte)
  }
}

/// The return type of [`percent_encode`] and [`utf8_percent_encode`].
#[derive(Clone, Copy, Debug)]
pub struct PercentEncode<'bytes> {
  bytes: &'bytes [u8],
  ascii_set: AsciiSet,
}

impl<'bytes> PercentEncode<'bytes> {
  /// New instance that will use `ascii_set` to guide which elements should be encoded.
  #[inline]
  pub const fn new(bytes: &'bytes [u8], ascii_set: AsciiSet) -> Self {
    Self { bytes, ascii_set }
  }
}

impl<'bytes> Iterator for PercentEncode<'bytes> {
  type Item = &'bytes [u8];

  #[inline]
  fn next(&mut self) -> Option<&'bytes [u8]> {
    let (&byte, remaining) = self.bytes.split_first()?;
    if self.ascii_set.should_percent_encode(byte) {
      self.bytes = remaining;
      Some(percent_encode_str(byte).as_bytes())
    } else {
      for (idx, local_byte) in remaining.iter().copied().enumerate() {
        if self.ascii_set.should_percent_encode(local_byte) {
          let (left, right) = self.bytes.split_at_checked(idx.wrapping_add(1))?;
          self.bytes = right;
          return Some(left);
        }
      }
      let rslt = self.bytes;
      self.bytes = &[][..];
      Some(rslt)
    }
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    if self.bytes.is_empty() { (0, Some(0)) } else { (1, Some(self.bytes.len())) }
  }
}

/// The return type of [`percent_decode`].
#[derive(Clone, Copy, Debug)]
pub struct PercentDecode<'bytes> {
  bytes: &'bytes [u8],
}

impl<'bytes> PercentDecode<'bytes> {
  /// New instance
  #[inline]
  pub const fn new(bytes: &'bytes [u8]) -> Self {
    Self { bytes }
  }

  /// If the inner bytes have a special character, decodes everything into `vector` returning
  /// `true`. Otherwise, returns `false` leaving `vector` untouched.
  #[inline]
  pub fn decode(self, vector: &mut Vector<u8>) -> crate::Result<bool> {
    vector.reserve(self.bytes.len())?;
    let mut bytes = self.bytes;
    let mut idx: usize = 0;
    let decoded_byte = 'unmodified: {
      while let [first, rest @ ..] = bytes {
        bytes = rest;
        if *first != b'%' {
          idx = idx.wrapping_add(1);
          continue;
        }
        let Some(byte) = manage_percent_char(&mut bytes) else {
          continue;
        };
        break 'unmodified byte;
      }
      return Ok(false);
    };
    let normal_bytes = self.bytes.get(..idx).unwrap_or_default();
    let _ = vector.extend_from_copyable_slices([normal_bytes, &[decoded_byte][..]])?;
    while let [byte, rest @ ..] = bytes {
      bytes = rest;
      vector.push(if *byte == b'%' {
        manage_percent_char(&mut bytes).unwrap_or(*byte)
      } else {
        *byte
      })?;
    }
    Ok(true)
  }
}

#[inline]
fn manage_percent_char(bytes: &mut &[u8]) -> Option<u8> {
  let [a, b, rest @ ..] = bytes else {
    return None;
  };
  let c = u8::try_from(char::from(*a).to_digit(16)?).ok()?;
  let d = u8::try_from(char::from(*b).to_digit(16)?).ok()?;
  *bytes = rest;
  Some(c.wrapping_mul(16).wrapping_add(d))
}

#[inline]
fn percent_encode_str(byte: u8) -> &'static str {
  static TABLE: &[u8; 768] = b"\
    %00%01%02%03%04%05%06%07%08%09%0A%0B%0C%0D%0E%0F\
    %10%11%12%13%14%15%16%17%18%19%1A%1B%1C%1D%1E%1F\
    %20%21%22%23%24%25%26%27%28%29%2A%2B%2C%2D%2E%2F\
    %30%31%32%33%34%35%36%37%38%39%3A%3B%3C%3D%3E%3F\
    %40%41%42%43%44%45%46%47%48%49%4A%4B%4C%4D%4E%4F\
    %50%51%52%53%54%55%56%57%58%59%5A%5B%5C%5D%5E%5F\
    %60%61%62%63%64%65%66%67%68%69%6A%6B%6C%6D%6E%6F\
    %70%71%72%73%74%75%76%77%78%79%7A%7B%7C%7D%7E%7F\
    %80%81%82%83%84%85%86%87%88%89%8A%8B%8C%8D%8E%8F\
    %90%91%92%93%94%95%96%97%98%99%9A%9B%9C%9D%9E%9F\
    %A0%A1%A2%A3%A4%A5%A6%A7%A8%A9%AA%AB%AC%AD%AE%AF\
    %B0%B1%B2%B3%B4%B5%B6%B7%B8%B9%BA%BB%BC%BD%BE%BF\
    %C0%C1%C2%C3%C4%C5%C6%C7%C8%C9%CA%CB%CC%CD%CE%CF\
    %D0%D1%D2%D3%D4%D5%D6%D7%D8%D9%DA%DB%DC%DD%DE%DF\
    %E0%E1%E2%E3%E4%E5%E6%E7%E8%E9%EA%EB%EC%ED%EE%EF\
    %F0%F1%F2%F3%F4%F5%F6%F7%F8%F9%FA%FB%FC%FD%FE%FF\
  ";
  let idx = usize::from(byte).wrapping_mul(3);
  let slice = TABLE.get(idx..idx.wrapping_add(3)).unwrap_or_default();
  // SAFETY: TABLE is ascii-only
  unsafe { str::from_utf8_unchecked(slice) }
}

#[cfg(test)]
mod tests {
  use crate::misc::{AsciiSet, PercentDecode, PercentEncode, Vector};

  #[test]
  fn decode() {
    let decoded = "y+DvKRKG/sTPjjmItrMFJZcCE/MBi5rlXPXsNA==";
    let encoded = "y%2BDvKRKG%2FsTPjjmItrMFJZcCE%2FMBi5rlXPXsNA%3D%3D";
    let mut buffer = Vector::new();
    let _ = PercentDecode::new(encoded.as_bytes()).decode(&mut buffer).unwrap();
    assert_eq!(buffer.as_slice(), decoded.as_bytes());
  }

  #[test]
  fn encode() {
    let mut buffer = Vector::new();
    for elem in PercentEncode::new(b"hello world?", AsciiSet::NON_ALPHANUMERIC) {
      buffer.extend_from_copyable_slice(elem).unwrap();
    }
    assert_eq!(buffer.as_ref(), b"hello%20world%3F");
  }
}
