use crate::{
  collection::Vector,
  misc::{EnvVars, FromVars, find_file, str_rsplit_once1, str_split_once1},
};
use alloc::string::String;
use core::str;
use std::{
  env, fs,
  io::{BufRead as _, BufReader, Read},
  path::Path,
};

impl<T> EnvVars<T>
where
  T: FromVars,
{
  /// Tries reading from [`Self::from_process`] and then fallbacks to [`Self::from_nearest_env_file`].
  ///
  /// Intended for development purposes.
  #[inline]
  pub fn from_available() -> crate::Result<Self> {
    if let Ok(elem) = Self::from_process() {
      return Ok(elem);
    }
    Self::from_nearest_env_file()
  }

  /// Constructs itself through the deserialization of a literal `.env` data.
  ///
  /// Intended for debugging or tests.
  #[inline]
  pub fn from_env_data(data: &[u8]) -> crate::Result<Self> {
    Ok(Self(T::from_vars(env(data)?)?))
  }

  /// Constructs itself through the deserialization of the passed `.env` file.
  ///
  /// Intended for development purposes.
  #[inline]
  pub fn from_env_path<P>(path: P) -> crate::Result<Self>
  where
    P: AsRef<Path>,
  {
    Ok(Self(T::from_vars(env(fs::File::open(path)?)?)?))
  }

  /// Tries to find an `.env` file starting at the current location until the root directory.
  ///
  /// Intended for development purposes.
  #[inline]
  pub fn from_nearest_env_file() -> crate::Result<Self> {
    let mut buffer = env::current_dir()?;
    find_file(&mut buffer, Path::new(".env"))?;
    Ok(Self(T::from_vars(env(fs::File::open(buffer)?)?)?))
  }

  /// Constructs itself according to all the environment variables of the current process.
  #[inline]
  pub fn from_process() -> crate::Result<Self> {
    Ok(Self(T::from_vars(env::vars())?))
  }

  /// Unwraps `T`.
  #[inline]
  pub fn finish(self) -> T {
    self.0
  }
}

#[expect(clippy::ref_patterns, reason = "false-positive")]
fn env<R>(read: R) -> crate::Result<Vector<(String, String)>>
where
  R: Read,
{
  let buffer = &mut String::new();
  let reader = &mut BufReader::new(read);
  let mut vars = Vector::new();
  loop {
    if reader.read_line(buffer)? == 0 {
      break;
    }
    let buffer_ref = buffer.trim();
    if buffer_ref.is_empty() || buffer_ref.starts_with('#') {
      buffer.clear();
      continue;
    }
    let Some((key, value)) = str_split_once1(buffer_ref, b'=') else {
      buffer.clear();
      continue;
    };
    let key_trimmed = key.trim_end().into();
    let value_trimmed = value.trim_start();
    if let &[delimiter @ (b'\'' | b'"'), ref value_after_del @ ..] = value_trimmed.as_bytes() {
      let diff = value.len().wrapping_sub(value_trimmed.len());
      let value_begin = key.len().wrapping_add(1).wrapping_add(diff).wrapping_add(1);
      if let &[ref value_surrounded @ .., last] = value_after_del {
        if delimiter == last {
          // SAFETY: The cut of surrounding quotes don't invalidate UTF-8
          let value_final = unsafe { str::from_utf8_unchecked(value_surrounded) };
          vars.push((key_trimmed, unescape(value_final)))?;
        } else {
          process_multiline(buffer, reader, delimiter, key_trimmed, value_begin, &mut vars)?;
        }
      } else {
        process_multiline(buffer, reader, delimiter, key_trimmed, value_begin, &mut vars)?;
      }
    } else {
      vars.push((key_trimmed, unescape(strip_ending_comment(value_trimmed))))?;
    }

    buffer.clear();
  }
  Ok(vars)
}

fn process_multiline<R>(
  buffer: &mut String,
  buf_reader: &mut BufReader<R>,
  delimiter: u8,
  key_trimmed: String,
  value_begin: usize,
  vars: &mut Vector<(String, String)>,
) -> crate::Result<()>
where
  R: Read,
{
  let mut ends_with_delimiter = false;
  let actual_buffer_data = loop {
    if buf_reader.read_line(buffer)? == 0 {
      break buffer.as_str();
    }
    let trimmed = buffer.trim_end();
    if trimmed.ends_with(char::from(delimiter)) {
      ends_with_delimiter = true;
      break trimmed;
    }
  };
  let mut value_all = actual_buffer_data.get(value_begin..).unwrap_or_default();
  if !ends_with_delimiter {
    value_all = strip_ending_comment(value_all);
  }
  let mut value_final = unescape(value_all);
  if Some(char::from(delimiter)) != value_final.pop() {
    return Err(crate::Error::MissingVarQuote(key_trimmed.into()));
  }
  vars.push((key_trimmed, value_final))?;
  Ok(())
}

fn strip_ending_comment(value: &str) -> &str {
  if let Some((lhs, _)) = str_rsplit_once1(value, b'#') { lhs.trim_end() } else { value }
}

fn unescape(str: &str) -> String {
  let mut rslt = String::with_capacity(str.len());
  let mut chars = str.chars();
  while let Some(ch) = chars.next() {
    if ch == '\\' {
      match chars.next() {
        Some('"') => rslt.push('"'),
        Some('\'') => rslt.push('\''),
        Some('\\') | None => rslt.push('\\'),
        Some('n') => rslt.push('\n'),
        Some('r') => rslt.push('\r'),
        Some('t') => rslt.push('\t'),
        Some(other) => {
          rslt.push('\\');
          rslt.push(other);
        }
      }
    } else {
      rslt.push(ch);
    }
  }
  rslt
}

#[cfg(test)]
mod tests {
  use crate::misc::env_vars::std::env;

  #[test]
  fn basic() {
    let data = "HOST='localhost'\nPORT=8080\n Comment\nNAME=\"foo\"";
    let result = env(data.as_bytes()).unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], ("HOST".into(), "localhost".into()));
    assert_eq!(result[1], ("PORT".into(), "8080".into()));
    assert_eq!(result[2], ("NAME".into(), "foo".into()));
  }

  #[test]
  fn comments() {
    {
      let data = "PORT=8080 # The server port";
      let result = env(data.as_bytes()).unwrap();
      assert_eq!(result.len(), 1);
      assert_eq!(result[0], ("PORT".into(), "8080".into()));
    }
    {
      let data = "PORT='8080' # The server port";
      let result = env(data.as_bytes()).unwrap();
      assert_eq!(result.len(), 1);
      assert_eq!(result[0], ("PORT".into(), "8080".into()));
    }
    {
      let data = "PORT='\n80\n80\n' # The server port";
      let result = env(data.as_bytes()).unwrap();
      assert_eq!(result.len(), 1);
      assert_eq!(result[0], ("PORT".into(), "\n80\n80\n".into()));
    }
  }

  #[test]
  fn escaped_quotes() {
    let data = r#"JSON="{\"a\":1}""#;
    let result = env(data.as_bytes()).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], ("JSON".into(), r#"{"a":1}"#.into()));
  }

  #[test]
  fn multiline_with_trailing_newline() {
    let data = "FOO=\"Line 1\nLine 2\"\nNEXT=bar";
    let result = env(data.as_bytes()).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], ("FOO".into(), "Line 1\nLine 2".into()));
    assert_eq!(result[1], ("NEXT".into(), "bar".into()));
  }

  #[test]
  fn new_lines() {
    {
      let data = "FOO='bar\nbaz'";
      let result = env(data.as_bytes()).unwrap();
      assert_eq!(result.len(), 1);
      assert_eq!(result[0], ("FOO".into(), "bar\nbaz".into()));
    }
    {
      let data = "FOO='
        bar
        baz
      '";
      let result = env(data.as_bytes()).unwrap();
      assert_eq!(result.len(), 1);
      assert_eq!(result[0], ("FOO".into(), "\n        bar\n        baz\n      ".into()));
    }
  }

  #[test]
  fn unclosed_variable() {
    let data = "FOO='bar";
    assert!(env(data.as_bytes()).is_err());
  }

  #[test]
  fn with_value_spaces() {
    let data = "FOO=\"  bar\"";
    let result = env(data.as_bytes()).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], ("FOO".into(), "  bar".into()));
  }
}
