use crate::collection::ShortStrU8;
use core::panic::Location;

/// Error with additional information about location
#[derive(Debug)]
pub struct ErrorInfo<E> {
  /// Column
  pub column: u16,
  /// Error
  pub error: E,
  /// File
  pub file: ShortStrU8<'static>,
  /// Line
  pub line: u16,
}

impl<E> ErrorInfo<E> {
  /// Automatically fills fields related to location.
  #[inline]
  #[track_caller]
  pub fn from_error(error: E) -> Self {
    let location = Location::caller();
    let column = if cfg!(feature = "error-loc") {
      location.column().try_into().unwrap_or(u16::MAX)
    } else {
      0
    };
    let file = if cfg!(feature = "error-loc") {
      ShortStrU8::new_truncated_u8(location.file())
    } else {
      ShortStrU8::default()
    };
    let line =
      if cfg!(feature = "error-loc") { location.line().try_into().unwrap_or(u16::MAX) } else { 0 };
    Self { column, error, file, line }
  }
}
