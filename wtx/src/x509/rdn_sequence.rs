use crate::{collection::Vector, x509::RelativeDistinguishedName};

/// An ordered sequence of Relative Distinguished Names forming a full DN.
pub type RdnSequence<'bytes> = Vector<RelativeDistinguishedName<'bytes>>;
