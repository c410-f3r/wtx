use crate::{
  de::{Decode, Encode},
  misc::{Lease, SuffixWriterMut},
  tls::{NamedGroup, TlsError, de::De, misc::u16_chunk},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KeyShareEntry<'any> {
  pub(crate) group: NamedGroup,
  pub(crate) opaque: &'any [u8],
}

impl<'any> KeyShareEntry<'any> {
  pub(crate) fn new(group: NamedGroup, opaque: &'any [u8]) -> Self {
    Self { group, opaque }
  }
}

impl<'de> Decode<'de, De> for KeyShareEntry<'de> {
  #[inline]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let group = NamedGroup::decode(dw)?;
    let opaque = u16_chunk(dw, TlsError::InvalidKeyShareEntry, |el| Ok(*el))?;
    Ok(Self { group, opaque })
  }
}

impl<'de> Encode<De> for KeyShareEntry<'de> {
  #[inline]
  fn encode(&self, ew: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    ew.extend_from_slices([
      &u16::from(self.group).to_be_bytes(),
      &u16::try_from(self.opaque.lease().len())?.to_be_bytes(),
      self.opaque.lease(),
    ])?;
    Ok(())
  }
}
