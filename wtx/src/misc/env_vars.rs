#[cfg(feature = "std")]
mod std;

use alloc::string::String;

/// Allows the interactive reading of environment variables.
#[derive(Debug)]
pub struct EnvVars<T>(T);

/// Constructs itself using a set of `(key, value)` string pairs.
pub trait FromVars: Sized {
  /// See [`FromVars`].
  fn from_vars(vars: impl IntoIterator<Item = (String, String)>) -> crate::Result<Self>;
}
