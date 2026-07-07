use crate::{
  collections::{ArrayStringU8, ArrayVectorCopy},
  misc::{ExtUtf8Error, from_utf8_ext},
};
use core::str;

/// Completion error
#[derive(Clone, Copy, Debug)]
pub enum CompletionErr {
  /// It is impossible to verify the string
  HasInvalidBytes,
  /// More bytes are needed to verify the string.
  InsufficientInput,
}

/// Bytes that can or can not represent an incomplete UTF-8 character.
#[derive(Clone, Copy, Debug)]
pub struct PartialChar(ArrayVectorCopy<u8, 4>);

impl PartialChar {
  #[inline]
  pub(crate) fn new(bytes: &[u8]) -> Option<Self> {
    if bytes.len() > 4 {
      return None;
    }
    let mut buffer = ArrayVectorCopy::new();
    let _rslt = buffer.extend_from_copyable_slice(bytes);
    Some(Self(buffer))
  }

  /// Tries to join the current set of bytes with the provided `bytes` to form a valid UTF-8 character.
  #[inline]
  fn complete<'bytes>(&mut self, bytes: &'bytes [u8]) -> (Result<(), CompletionErr>, &'bytes [u8]) {
    let (consumed, complete_err) = self.push_to_build_valid_char(bytes);
    let remaining = bytes.get(consumed..).unwrap_or_default();
    match complete_err {
      None => (Ok(()), remaining),
      Some(elem) => (Err(elem), remaining),
    }
  }

  #[inline]
  fn push_to_build_valid_char(&mut self, bytes: &[u8]) -> (usize, Option<CompletionErr>) {
    let initial_len: usize = self.0.len().into();
    let to_write_len = {
      let unwritten: usize = self.0.remaining().into();
      let to_write_len = unwritten.min(bytes.len());
      let _rslt = self.0.extend_from_copyable_slice(bytes.get(..to_write_len).unwrap_or_default());
      to_write_len
    };
    let new_bytes = {
      let len = initial_len.wrapping_add(to_write_len);
      self.0.get(..len).unwrap_or_default()
    };
    if let Err(err) = crate::misc::from_utf8_std(new_bytes) {
      if err.valid_up_to > 0 {
        self.0.truncate(err.valid_up_to.try_into().unwrap_or_default());
        (err.valid_up_to.saturating_sub(initial_len), None)
      } else {
        match err.error_len {
          None => (to_write_len, Some(CompletionErr::InsufficientInput)),
          Some(_) => (to_write_len, Some(CompletionErr::HasInvalidBytes)),
        }
      }
    } else {
      (to_write_len, None)
    }
  }
}

/// Processes a chunk of bytes from a continuous UTF-8 stream, seamlessly handling characters that
/// are split across chunk boundaries.
#[inline]
pub fn process_utf8_stream<'nb>(
  partial_char: &mut Option<PartialChar>,
  bytes: &'nb [u8],
) -> crate::Result<(ArrayStringU8<4>, &'nb str)> {
  let mut lhs = ArrayStringU8::new();
  let tail = if let Some(mut incomplete) = partial_char.take() {
    let (rslt, tail) = incomplete.complete(bytes);
    match rslt {
      Err(CompletionErr::HasInvalidBytes) => return Err(crate::Error::InvalidUTF8),
      Err(CompletionErr::InsufficientInput) => {
        *partial_char = Some(incomplete);
        return Ok((ArrayStringU8::new(), ""));
      }
      Ok(_) => {
        // SAFETY: a successful `complete` ensures that the internal buffer is UTF-8
        let _rslt = lhs.push_str(unsafe { str::from_utf8_unchecked(&incomplete.0) });
        tail
      }
    }
  } else {
    bytes
  };
  match from_utf8_ext(tail) {
    Err(ExtUtf8Error::Incomplete(el)) => {
      let idx = tail.len().saturating_sub(el.0.len().into());
      *partial_char = Some(el);
      // SAFETY: `from_utf8_ext` ensures that everything but the last incomplete characters
      //         are UTF-8
      let rhs = unsafe { str::from_utf8_unchecked(tail.get(..idx).unwrap_or_default()) };
      Ok((lhs, rhs))
    }
    Err(ExtUtf8Error::Invalid) => Err(crate::Error::InvalidUTF8),
    Ok(rhs) => Ok((lhs, rhs)),
  }
}
