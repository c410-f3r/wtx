/// Iterator wrapper to work around coherence rules.
#[derive(Debug)]
pub struct IterWrapper<I>(
  /// Iterator
  pub I,
)
where
  I: Iterator;
