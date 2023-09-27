mod incomplete_utf8_char;
mod traits;
pub(crate) mod uri_parts;
mod utf8_errors;

use core::ops::Range;
pub(crate) use incomplete_utf8_char::{CompleteErr, IncompleteUtf8Char};
pub(crate) use traits::{Expand, SingleTypeStorage};
pub(crate) use utf8_errors::{ExtUtf8Error, StdUtf8Error};

pub(crate) fn from_utf8_opt(bytes: &[u8]) -> Option<&str> {
  #[cfg(feature = "simdutf8")]
  return simdutf8::basic::from_utf8(bytes).ok();
  #[cfg(not(feature = "simdutf8"))]
  return core::str::from_utf8(bytes).ok();
}

pub(crate) fn from_utf8_ext_rslt(bytes: &[u8]) -> Result<&str, ExtUtf8Error> {
  let err = match from_utf8_std_rslt(bytes) {
    Ok(elem) => return Ok(elem),
    Err(error) => error,
  };
  let (_valid_bytes, after_valid) = bytes.split_at(err.valid_up_to);
  match err.error_len {
    None => Err(ExtUtf8Error::Incomplete {
      incomplete_ending_char: {
        let opt = IncompleteUtf8Char::new(after_valid);
        opt.ok_or(ExtUtf8Error::Invalid)?
      },
    }),
    Some(_) => Err(ExtUtf8Error::Invalid),
  }
}

pub(crate) fn from_utf8_std_rslt(bytes: &[u8]) -> Result<&str, StdUtf8Error> {
  #[cfg(feature = "simdutf8")]
  return simdutf8::compat::from_utf8(bytes).map_err(|element| StdUtf8Error {
    valid_up_to: element.valid_up_to(),
    error_len: element.error_len(),
  });
  #[cfg(not(feature = "simdutf8"))]
  return core::str::from_utf8(bytes).map_err(|element| StdUtf8Error {
    valid_up_to: element.valid_up_to(),
    error_len: element.error_len(),
  });
}

#[cfg(test)]
pub(crate) fn _tracing() -> crate::Result<impl tracing_subscriber::util::SubscriberInitExt> {
  use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter};
  let env_filter = EnvFilter::from_default_env();
  let tracing_tree = tracing_tree::HierarchicalLayer::default()
    .with_writer(std::io::stdout)
    .with_indent_lines(true)
    .with_indent_amount(2)
    .with_thread_names(false)
    .with_thread_ids(true)
    .with_verbose_exit(false)
    .with_verbose_entry(false)
    .with_targets(true);
  let subscriber = tracing_subscriber::Registry::default().with(env_filter).with(tracing_tree);
  Ok(subscriber)
}

pub(crate) fn _trim(bytes: &[u8]) -> &[u8] {
  _trim_end(_trim_begin(bytes))
}

pub(crate) fn _truncated_slice<T>(slice: &[T], range: Range<usize>) -> &[T] {
  let start = range.start;
  let end = range.end.min(slice.len());
  slice.get(start..end).unwrap_or_default()
}

fn _trim_begin(mut bytes: &[u8]) -> &[u8] {
  while let [first, rest @ ..] = bytes {
    if first.is_ascii_whitespace() {
      bytes = rest;
    } else {
      break;
    }
  }
  bytes
}

fn _trim_end(mut bytes: &[u8]) -> &[u8] {
  while let [rest @ .., last] = bytes {
    if last.is_ascii_whitespace() {
      bytes = rest;
    } else {
      break;
    }
  }
  bytes
}
