use crate::{
  collection::Vector,
  misc::{EnvVars, FromVars, str_split_once1},
};
use alloc::string::String;
use core::str;
use std::{
  env, fs,
  io::{self, BufRead as _, BufReader, Read},
  path::{Path, PathBuf},
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

  /// Constructs `T` through the deserialization of a literal `.env` data.
  ///
  /// Intended for debugging or tests.
  #[inline]
  pub fn from_env_data(data: &[u8]) -> crate::Result<Self> {
    Ok(Self(T::from_vars(env(data)?)?))
  }

  /// Constructs `T` through the deserialization of the passed `.env` file.
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

  /// Constructs `T` according to all the environment variables of the current process.
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

fn env<R>(read: R) -> crate::Result<Vector<(String, String)>>
where
  R: Read,
{
  let mut buf_reader = BufReader::new(read);
  let mut buffer = String::new();
  let mut vars = Vector::new();
  loop {
    if buf_reader.read_line(&mut buffer)? == 0 {
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
    vars.push((into_string(key), into_string(value)))?;
    buffer.clear();
  }
  Ok(vars)
}

fn find_file(buffer: &mut PathBuf, path: &Path) -> io::Result<()> {
  buffer.push(path);
  match fs::metadata(&buffer) {
    Ok(elem) => {
      if elem.is_file() {
        return Ok(());
      }
    }
    Err(err) => {
      if err.kind() != io::ErrorKind::NotFound {
        return Err(err);
      }
    }
  }
  let _ = buffer.pop();
  if buffer.pop() {
    find_file(buffer, path)
  } else {
    Err(io::Error::new(io::ErrorKind::NotFound, "`.env` file not found"))
  }
}

fn into_string(str: &str) -> String {
  let mut bytes = str.trim().as_bytes();
  if let [b'\'', rest @ .., b'\''] | [b'"', rest @ .., b'"'] = bytes {
    bytes = rest;
  }
  // SAFETY: The cut of surrounding quotes don't invalidate UTF-8
  String::from(unsafe { str::from_utf8_unchecked(bytes) })
}

#[cfg(test)]
mod tests {
  use crate::misc::env_vars::std::env;

  #[test]
  fn basic_env() {
    let data = "HOST='localhost'\nPORT=8080\n Comment\nNAME=\"foo\"";
    let result = env(data.as_bytes()).unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], ("HOST".into(), "localhost".into()));
    assert_eq!(result[1], ("PORT".into(), "8080".into()));
    assert_eq!(result[2], ("NAME".into(), "foo".into()));
  }
}
