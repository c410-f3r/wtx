/// Router paths
#[derive(Debug)]
pub struct PathParams<T> {
  pub(crate) full_path: &'static str,
  pub(crate) value: T,
}

impl<T> PathParams<T> {
  /// Creates a new instance
  #[inline]
  pub const fn new(full_path: &'static str, value: T) -> Self {
    Self { full_path, value }
  }
}
