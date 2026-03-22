use alloc::string::String;

/// Constructs itself using a set of `(key, value)` string pairs.
pub trait FromVars: Sized {
  /// See [`FromVars`].
  fn from_vars(vars: impl IntoIterator<Item = (String, String)>) -> crate::Result<Self>;
}
