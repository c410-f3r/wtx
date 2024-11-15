/// Wrapper used to work around coherence rules.
#[derive(Debug)]
pub struct Wrapper<T>(
  /// Element
  pub T,
)
where
  T: Iterator;
