use crate::{
  collection::Vector,
  misc::{EnvVars, FromVars, find_file, str_rsplit_once1, str_split_once1},
};
use alloc::string::String;
use core::{fmt::Write as _, str};
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
  /// `others` can be used to pass sensible variables that are locally available for tests or
  /// development but are retrieved by other means in production.
  ///
  /// For example, in a system that needs two variables `DATABASE_URI` and `ROOT_CA` to work:
  ///
  /// * `development`: Both elements are inside an `.env` file.
  /// * `continuous integration`: Both elements are passed as environment variables.
  /// * `production`: `ROOT_CA` is passed as an environment variable and `DATABASE_URI` is
  ///   fetched from `HashiCorp` Vault.
  ///
  /// ```ignore
  /// use wtx::{
  ///   collection::Vector,
  ///   misc::{EnvVars, Secret},
  ///   rng::{ChaCha20, CryptoSeedableRng},
  /// };
  ///
  /// #[derive(wtx::FromVars)]
  /// struct Vars {
  ///   #[from_vars(map_secret)]
  ///   database_uri: Secret,
  ///   root_ca: String,
  /// }
  ///
  /// fn map_secret(var: String) -> wtx::Result<Secret> {
  ///   let mut rng = ChaCha20::from_std_random()?;
  ///   let secret_context = SecretContext::new(&mut rng)?;
  ///   Ok(Secret::new(&mut var.into_bytes(), &mut rng, secret_context)?)
  /// }
  ///
  /// async fn fetch_database_uri() -> String {
  ///   "Some secret value fetched from HashiCorp Vault".into()
  /// }
  ///
  /// async fn manage_vars() -> Vars {
  ///   let mut others = Vector::new();
  ///   if cfg!(feature = "production") {
  ///     others.push(("DATABASE_URI".into(), fetch_database_uri().await)).unwrap();
  ///   }
  ///   EnvVars::from_available(others).unwrap().finish()
  /// }
  /// ```
  ///
  /// Beware of accidental deploys of `.env` files because this method will read them if the
  /// initial [`Self::from_process`] fails.
  #[inline]
  pub fn from_available(others: impl IntoIterator<Item = (String, String)>) -> crate::Result<Self> {
    let err0 = match Self::from_process(others) {
      Ok(elem) => return Ok(elem),
      Err(err) => err,
    };
    let err1 = match Self::from_nearest_env_file() {
      Ok(elem) => return Ok(elem),
      Err(err) => err,
    };
    let mut error = String::new();
    error.write_fmt(format_args!("Errors: {err0}, {err1}"))?;
    Err(crate::Error::NoAvailableVars(error.try_into()?))
  }

  /// Constructs itself through the deserialization of a literal `.env` file data.
  ///
  /// Intended for debugging or tests.
  #[inline]
  pub fn from_env_data(data: &[u8]) -> crate::Result<Self> {
    let vector = env(data)?;
    Ok(Self(T::from_vars(vector)?))
  }

  /// Constructs itself through the deserialization of the passed `.env` file path.
  ///
  /// Intended for development purposes.
  #[inline]
  pub fn from_env_path<P>(path: P) -> crate::Result<Self>
  where
    P: AsRef<Path>,
  {
    let vector = env(fs::File::open(path)?)?;
    Ok(Self(T::from_vars(vector)?))
  }

  /// Tries to find an `.env` file starting at the current location until the root directory.
  ///
  /// Intended for development purposes.
  #[inline]
  pub fn from_nearest_env_file() -> crate::Result<Self> {
    let mut buffer = env::current_dir()?;
    find_file(&mut buffer, Path::new(".env"))?;
    let vector = env(fs::File::open(buffer)?)?;
    Ok(Self(T::from_vars(vector)?))
  }

  /// Constructs itself according to all the environment variables of the current process.
  ///
  /// See [`Self::from_available`] to understand the meaning of `others`.
  #[inline]
  pub fn from_process(others: impl IntoIterator<Item = (String, String)>) -> crate::Result<Self> {
    Ok(Self(T::from_vars(env::vars().chain(others))?))
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
    return Err(crate::Error::MissingVarQuote(key_trimmed.try_into()?));
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
