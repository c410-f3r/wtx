use crate::{
  collections::ArrayVectorU8,
  misc::Lease,
  x509::{CvEndEntity, CvIntermediate, CvTrustAnchor, MAX_INTERMEDIATES},
};

/// The path between an end entity and a trust anchor. These entities are connected by the
/// intermediates.
#[derive(Debug)]
pub struct VerifiedPath<'any, B>
where
  &'any [u8]: Lease<[u8]>,
{
  end_entity: &'any CvEndEntity<&'any [u8]>,
  intermediates: ArrayVectorU8<&'any CvIntermediate<&'any [u8]>, MAX_INTERMEDIATES>,
  trust_anchor: &'any CvTrustAnchor<B>,
}

impl<'any, B> VerifiedPath<'any, B>
where
  &'any [u8]: Lease<[u8]>,
  B: Lease<[u8]>,
{
  pub(crate) fn new(
    end_entity: &'any CvEndEntity<&'any [u8]>,
    intermediates: ArrayVectorU8<&'any CvIntermediate<&'any [u8]>, MAX_INTERMEDIATES>,
    trust_anchor: &'any CvTrustAnchor<B>,
  ) -> Self {
    Self { end_entity, intermediates, trust_anchor }
  }

  /// The provided certificate that started the validation.
  #[inline]
  pub fn end_entity(&self) -> &'any CvEndEntity<&'any [u8]> {
    self.end_entity
  }

  /// Certificates that connect the end entity and the trust anchor.
  #[inline]
  pub fn intermediates(&self) -> &[&'any CvIntermediate<&'any [u8]>] {
    &self.intermediates
  }

  /// Mutable version of [`Self::intermediates`].
  pub(crate) fn intermediates_mut(
    &mut self,
  ) -> &mut ArrayVectorU8<&'any CvIntermediate<&'any [u8]>, MAX_INTERMEDIATES> {
    &mut self.intermediates
  }

  /// See [`CvTrustAnchor`].
  #[inline]
  pub fn trust_anchor(&self) -> &'any CvTrustAnchor<B> {
    self.trust_anchor
  }

  /// Mutable version of [`Self::trust_anchor`].
  pub(crate) fn trust_anchor_mut(&mut self) -> &mut &'any CvTrustAnchor<B> {
    &mut self.trust_anchor
  }
}
