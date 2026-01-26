use crate::{
  de::{Decode, Encode},
  misc::Lease,
  tls::{
    NamedGroup, TlsError, de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper,
    misc::u16_chunk,
  },
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
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let group = NamedGroup::decode(dw)?;
    let opaque = if dw.is_hello_retry_request() {
      &[][..]
    } else {
      u16_chunk(dw, TlsError::InvalidKeyShareEntry, |el| Ok(el.bytes()))?
    };
    Ok(Self { group, opaque })
  }
}

impl<'de> Encode<De> for KeyShareEntry<'de> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    if ew.is_hello_retry_request() {
      ew.buffer().extend_from_slice(&u16::from(self.group).to_be_bytes())?;
    } else {
      ew.buffer().extend_from_slices([
        &u16::from(self.group).to_be_bytes(),
        &u16::try_from(self.opaque.lease().len())?.to_be_bytes(),
        self.opaque.lease(),
      ])?;
    }
    Ok(())
  }
}
