use crate::{
  collection::ArrayVectorU8,
  x509::{CvEndEntity, CvIntermediate, CvTrustAnchor, MAX_INTERMEDIATES},
};

/// The path between an end entity and a trust anchor. These entities are connected by the
/// intermediates.
#[derive(Debug)]
pub struct VerifiedPath<'any, 'bytes> {
  end_entity: &'any CvEndEntity<'any, 'bytes>,
  intermediates: ArrayVectorU8<&'any CvIntermediate<'any, 'bytes>, MAX_INTERMEDIATES>,
  trust_anchor: &'any CvTrustAnchor<'bytes>,
}

impl<'any, 'bytes> VerifiedPath<'any, 'bytes> {
  pub(crate) fn new(
    end_entity: &'any CvEndEntity<'any, 'bytes>,
    intermediates: ArrayVectorU8<&'any CvIntermediate<'any, 'bytes>, MAX_INTERMEDIATES>,
    trust_anchor: &'any CvTrustAnchor<'bytes>,
  ) -> Self {
    Self { end_entity, intermediates, trust_anchor }
  }

  /// The provided certificate that started the validation.
  pub fn end_entity(&self) -> &'any CvEndEntity<'any, 'bytes> {
    self.end_entity
  }

  /// Certificates that connect the end entity and the trust anchor.
  pub fn intermediates(&self) -> &[&'any CvIntermediate<'any, 'bytes>] {
    &self.intermediates
  }

  /// Mutable version of [`Self::intermediates`].
  pub(crate) fn intermediates_mut(
    &mut self,
  ) -> &mut ArrayVectorU8<&'any CvIntermediate<'any, 'bytes>, MAX_INTERMEDIATES> {
    &mut self.intermediates
  }

  /// See [`CvTrustAnchor`].
  pub fn trust_anchor(&self) -> &'any CvTrustAnchor<'bytes> {
    self.trust_anchor
  }

  /// Mutable version of [`Self::trust_anchor`].
  pub(crate) fn trust_anchor_mut(&mut self) -> &mut &'any CvTrustAnchor<'bytes> {
    &mut self.trust_anchor
  }
}
