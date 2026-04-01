/// Semantic entry-point where certificates can be validated.
#[derive(Debug, PartialEq)]
pub struct EndEntityCert<C>(
  /// Generic Certificate
  pub C,
);
