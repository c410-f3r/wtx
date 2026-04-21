use crate::{
  collection::ArrayVectorU8,
  x509::{CvTrustAnchor, NameVector},
};
use hashbrown::HashMap;

/// Store of [`CvTrustAnchor`] certificates. Basically groups a bunch of trusted
/// certificates to perform a chain validation.
#[derive(Debug)]
pub struct TavStore<'bytes>(
  pub HashMap<NameVector<'bytes>, ArrayVectorU8<CvTrustAnchor<'bytes, 'bytes>, 4>>,
);

impl<'bytes> TavStore<'bytes> {}
