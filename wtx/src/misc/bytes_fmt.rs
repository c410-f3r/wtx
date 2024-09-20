use core::{
  fmt::{Display, Formatter},
  str,
};

/// A wrapper that allows the formatting of byte slices through [`Display`]. Should only
/// be used with vectors.
#[derive(Debug)]
pub struct BytesFmt<'bytes>(
  /// Bytes
  pub &'bytes [u8],
);

impl<'bytes> Display for BytesFmt<'bytes> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    for elem in self.0.iter().copied() {
      // SAFETY: `BytesFmt` is intended to be used with vectors.
      f.write_str(unsafe { str::from_utf8_unchecked(&[elem]) })?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::misc::{BytesFmt, Vector};
  use alloc::string::String;
  use core::fmt::Write;

  #[test]
  fn has_correct_outputs_for_string() {
    let slice = &[56, 58, 200];
    let mut elem = String::new();
    elem.write_fmt(format_args!("{}", BytesFmt(slice))).unwrap();
    assert_eq!(slice, elem.as_bytes());
  }

  #[test]
  fn has_correct_outputs_for_vector() {
    let slice = &[56, 58, 200];
    let mut elem = Vector::new();
    elem.write_fmt(format_args!("{}", BytesFmt(slice))).unwrap();
    assert_eq!(slice, elem.as_ref());
  }
}
