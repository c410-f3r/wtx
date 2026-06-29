/// Chain Validation - Policy Mode
///
/// Dictates non-configurable rules in chain validation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CvPolicyMode {
  /// Ignores some rules.
  #[default]
  Lenient,
  /// Tries to enforce as much policies as possible.
  Strict,
}

impl CvPolicyMode {
  /// Returns `true` if the cv policy mode is [`Self::Lenient`].
  #[inline]
  #[must_use]
  pub const fn is_lenient(&self) -> bool {
    matches!(self, Self::Lenient)
  }

  /// Returns `true` if the chain validation policy mode is [`Self::Strict`].
  #[inline]
  #[must_use]
  pub const fn is_strict(&self) -> bool {
    matches!(self, Self::Strict)
  }
}
