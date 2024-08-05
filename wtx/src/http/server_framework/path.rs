/// Router paths
#[derive(Debug)]
pub struct Path<T> {
  pub(crate) name: &'static str,
  pub(crate) value: T,
}

impl<T> Path<T> {
  /// Creates a new instance
  #[inline]
  pub fn new(name: &'static str, value: T) -> Self {
    Self { name, value }
  }
}
