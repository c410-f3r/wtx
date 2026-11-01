use crate::{
  calendar::{Datetime, Utc},
  x509::{
    CvPolicyMode,
    cv::{cv_crl_expiration::CvCrlExpiration, cv_evaluation_depth::CvEvaluationDepth},
    extensions::{ExtendedKeyUsage, KeyUsage},
  },
};

/// Chain Validation - Policy
///
/// Groups all configurable rule parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CvPolicy<C> {
  cep: CvCrlExpiration,
  crls: C,
  evaluation_depth: CvEvaluationDepth,
  extended_key_usage: ExtendedKeyUsage,
  key_usage: KeyUsage,
  mode: CvPolicyMode,
  validation_time: Datetime<Utc>,
}

impl<C> CvPolicy<C> {
  /// New instance with optioned parameters.
  #[inline]
  pub fn new(crls: C, validation_time: Datetime<Utc>) -> Self {
    Self {
      cep: CvCrlExpiration::Enforce,
      crls,
      evaluation_depth: CvEvaluationDepth::Chain(8),
      extended_key_usage: ExtendedKeyUsage::default(),
      key_usage: KeyUsage::default(),
      mode: CvPolicyMode::Strict,
      validation_time: validation_time.trunc_to_secs(),
    }
  }

  /// See [`CvCrl`](crate::x509::CvCrl).
  #[inline]
  pub const fn crls(&self) -> &C {
    &self.crls
  }

  /// Mutable version of [`Self::crls`].
  #[inline]
  pub const fn crls_mut(&mut self) -> &mut C {
    &mut self.crls
  }

  /// See [`ExtendedKeyUsage`].
  #[inline]
  pub const fn extended_key_usage(&self) -> &ExtendedKeyUsage {
    &self.extended_key_usage
  }

  /// Mutable version of [`Self::extended_key_usage`].
  #[inline]
  pub const fn extended_key_usage_mut(&mut self) -> &mut ExtendedKeyUsage {
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

  /// Maps all generic types
  #[inline]
  pub fn map<'this, NC>(&'this self, mut crls: impl FnMut(&'this C) -> NC) -> CvPolicy<NC> {
    CvPolicy {
      cep: self.cep,
      crls: crls(&self.crls),
      evaluation_depth: self.evaluation_depth,
      extended_key_usage: self.extended_key_usage,
      key_usage: self.key_usage,
      mode: self.mode,
      validation_time: self.validation_time,
    }
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

  /// Overwrites the validation time.
  #[inline]
  pub const fn set_validation_time(&mut self, value: Datetime<Utc>) {
    self.validation_time = value.trunc_to_secs();
  }

  /// No certificate can have an expiration time lesser than this value.
  #[inline]
  pub const fn validation_time(&self) -> &Datetime<Utc> {
    &self.validation_time
  }
}
