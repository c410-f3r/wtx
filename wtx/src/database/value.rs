/// Groups common methods that describe the input parameter of [crate::database::Decode::decode].
pub trait Value: Default {
  /// If the input represents a null element. Somethings it can be an empty set of bytes or
  /// something it can be prefix flag.
  fn is_null(&self) -> bool;
}

impl Value for () {
  #[inline]
  fn is_null(&self) -> bool {
    true
  }
}
