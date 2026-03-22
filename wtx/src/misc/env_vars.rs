#[cfg(feature = "std")]
mod std;

use crate::misc::FromVars;
use alloc::string::String;

/// Allows the interactive reading of environment variables.
#[derive(Debug)]
pub struct EnvVars<T>(T);

impl<T> EnvVars<T>
where
  T: FromVars,
{
  /// Constructs itself based on `vars`.
  ///
  /// Intended for debugging or tests.
  #[inline]
  pub fn from_iterator(vars: impl IntoIterator<Item = (String, String)>) -> crate::Result<Self> {
    Ok(Self(T::from_vars(vars)?))
  }
}
