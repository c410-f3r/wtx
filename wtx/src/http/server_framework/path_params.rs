/// Router paths
#[derive(Debug)]
pub struct PathParams<T> {
  pub(crate) name: &'static str,
  pub(crate) value: T,
}

impl<T> PathParams<T> {
  /// Creates a new instance
  #[inline]
  pub fn new(name: &'static str, value: T) -> Self {
    Self { name, value }
  }
}
