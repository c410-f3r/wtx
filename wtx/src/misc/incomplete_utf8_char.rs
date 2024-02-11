use crate::misc::from_utf8_std;

/// Completion error
#[derive(Debug)]
pub enum CompletionErr {
  /// It is impossible to verify the string
  HasInvalidBytes,
  /// More bytes are needed to verify the string.
  InsufficientInput,
}

/// Bytes that can or can not represent an incomplete UTF-8 character.
#[derive(Debug)]
pub struct IncompleteUtf8Char {
  buffer: [u8; 4],
  len: usize,
}

impl IncompleteUtf8Char {
  pub(crate) fn new(bytes: &[u8]) -> Option<Self> {
    let mut buffer = [0, 0, 0, 0];
    match bytes {
      [] => {}
      [a] => {
        buffer[0] = *a;
      }
      [a, b] => {
        buffer[0] = *a;
        buffer[1] = *b;
      }
      [a, b, c] => {
        buffer[0] = *a;
        buffer[1] = *b;
        buffer[2] = *c;
      }
      [a, b, c, d] => {
        buffer[0] = *a;
        buffer[1] = *b;
        buffer[2] = *c;
        buffer[3] = *d;
      }
      _ => return None,
    }
    Some(Self { buffer, len: bytes.len() })
  }

  /// Tries to join the current set of bytes with the provided `bytes` to form a valid UTF-8 character.
  #[inline]
  pub fn complete<'bytes>(
    &mut self,
    bytes: &'bytes [u8],
  ) -> (Result<(), CompletionErr>, &'bytes [u8]) {
    let (consumed, complete_err) = self.push_to_build_valid_char(bytes);
    let remaining = bytes.get(consumed..).unwrap_or_default();
    match complete_err {
      None => (Ok(()), remaining),
      Some(elem) => (Err(elem), remaining),
    }
  }

  fn push_to_build_valid_char(&mut self, bytes: &[u8]) -> (usize, Option<CompletionErr>) {
    let initial_len = self.len;
    let to_write_len = {
      let unwritten = self.buffer.get_mut(initial_len..).unwrap_or_default();
      let to_write_len = unwritten.len().min(bytes.len());
      unwritten
        .get_mut(..to_write_len)
        .unwrap_or_default()
        .copy_from_slice(bytes.get(..to_write_len).unwrap_or_default());
      to_write_len
    };
    let new_bytes = {
      let len = initial_len.wrapping_add(to_write_len);
      self.buffer.get(..len).unwrap_or_default()
    };
    if let Err(err) = from_utf8_std(new_bytes) {
      if err.valid_up_to > 0 {
        self.len = err.valid_up_to;
        (err.valid_up_to.wrapping_sub(initial_len), None)
      } else {
        match err.error_len {
          None => {
            self.len = new_bytes.len();
            (to_write_len, Some(CompletionErr::InsufficientInput))
          }
          Some(invalid_seq_len) => {
            self.len = invalid_seq_len;
            (invalid_seq_len.wrapping_sub(initial_len), Some(CompletionErr::HasInvalidBytes))
          }
        }
      }
    } else {
      self.len = new_bytes.len();
      (to_write_len, None)
    }
  }
}
