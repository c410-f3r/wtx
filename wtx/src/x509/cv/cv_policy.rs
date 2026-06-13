use crate::{
  calendar::{DateTime, Utc},
  x509::{
    CvCrl, CvPolicyMode,
    cv::{cv_crl_expiration::CvCrlExpiration, cv_evaluation_depth::CvEvaluationDepth},
    extensions::{ExtendedKeyUsage, KeyUsage},
  },
};

/// Chain Validation - Policy
///
/// Groups all configurable rule parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct CvPolicy<'any, 'bytes> {
  cep: CvCrlExpiration,
  crls: &'any [CvCrl<'any, 'bytes>],
  evaluation_depth: CvEvaluationDepth,
  extended_key_usage: &'any ExtendedKeyUsage,
  key_usage: KeyUsage,
  mode: CvPolicyMode,
  validation_time: DateTime<Utc>,
}

impl<'any, 'bytes> CvPolicy<'any, 'bytes> {
  /// New instance with optioned parameters.
  //
  // FIXME(STABLE): Use `::default()`
  pub const fn new() -> Self {
    Self {
      cep: CvCrlExpiration::Enforce,
      crls: &[],
      evaluation_depth: CvEvaluationDepth::Chain(8),
      extended_key_usage: const { &ExtendedKeyUsage::EMPTY },
      key_usage: KeyUsage::new((0, 0)),
      mode: CvPolicyMode::Strict,
      validation_time: DateTime::EPOCH,
    }
  }

  /// See [`CvCrl`].
  #[inline]
  pub const fn crls(&self) -> &'any [CvCrl<'any, 'bytes>] {
    self.crls
  }

  /// Mutable version of [`Self::crls`].
  #[inline]
  pub const fn crls_mut(&mut self) -> &mut &'any [CvCrl<'any, 'bytes>] {
    &mut self.crls
  }

  /// See [`ExtendedKeyUsage`].
  #[inline]
  pub const fn extended_key_usage(&self) -> &ExtendedKeyUsage {
    self.extended_key_usage
  }

  /// Mutable version of [`Self::extended_key_usage`].
  #[inline]
  pub const fn extended_key_usage_mut(&mut self) -> &mut &'any ExtendedKeyUsage {
    &mut self.extended_key_usage
  }

  /// See [`CvEvaluationDepth`].
  #[inline]
  pub const fn evaluation_depth(&self) -> CvEvaluationDepth {
    self.evaluation_depth
  }

  /// Mutable version of [`Self::evaluation_depth`].
  #[inline]
  pub const fn evaluation_depth_mut(&mut self) -> &mut CvEvaluationDepth {
    &mut self.evaluation_depth
  }

  /// See [`CvCrlExpiration`].
  #[inline]
  pub const fn expiration_policy(&self) -> CvCrlExpiration {
    self.cep
  }

  /// Mutable version of [`Self::expiration_policy`].
  #[inline]
  pub const fn expiration_policy_mut(&mut self) -> &mut CvCrlExpiration {
    &mut self.cep
  }

  /// See [`KeyUsage`].
  #[inline]
  pub const fn key_usage(&self) -> &KeyUsage {
    &self.key_usage
  }

  /// Mutable version of [`Self::key_usage`].
  #[inline]
  pub const fn key_usage_mut(&mut self) -> &mut KeyUsage {
    &mut self.key_usage
  }

  /// See [`CvPolicyMode`].
  #[inline]
  pub const fn mode(&self) -> CvPolicyMode {
    self.mode
  }

  /// Mutable version of [`Self::mode`].
  #[inline]
  pub const fn mode_mut(&mut self) -> &mut CvPolicyMode {
    &mut self.mode
  }

  /// Mutable version of [`Self::crls`].
  #[inline]
  pub const fn set_crls(&mut self, crls: &'any [CvCrl<'any, 'bytes>]) {
    self.crls = crls;
  }

  /// Mutable version of [`Self::validation_time`].
  #[inline]
  pub const fn set_validation_time(&mut self, value: DateTime<Utc>) {
    self.validation_time = value.trunc_to_sec();
  }

  /// No certificate can have an expiration time lesser than this value.
  #[inline]
  pub const fn validation_time(&self) -> &DateTime<Utc> {
    &self.validation_time
  }
}

impl<'any, 'bytes> Default for CvPolicy<'any, 'bytes> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}
