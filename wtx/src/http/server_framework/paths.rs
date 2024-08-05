/// Router paths
#[derive(Debug)]
pub struct Paths<P> {
  pub(crate) collection: P,
}

impl<P> Paths<P> {
  /// Creates a new instance
  #[inline]
  pub fn new(paths: P) -> Self {
    Self { collection: paths }
  }
}
