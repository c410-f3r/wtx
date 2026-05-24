use crate::{
  collection::ArrayVectorU8,
  x509::{CvTrustAnchor, Name},
};
use hashbrown::HashMap;

/// Store of [`CvTrustAnchor`] certificates. Basically groups a bunch of trusted
/// certificates to perform a chain validation.
#[derive(Debug)]
pub struct TavStore<'bytes>(pub HashMap<Name<'bytes>, ArrayVectorU8<CvTrustAnchor<'bytes>, 4>>);

impl TavStore<'_> {}
