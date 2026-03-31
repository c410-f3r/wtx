use crate::{collection::ArrayVectorU32, x509::AttributeTypeAndValue};

/// Unordered set of attribute type-value pairs.
pub type RelativeDistinguishedName<'bytes> = ArrayVectorU32<AttributeTypeAndValue<'bytes>, 2>;
