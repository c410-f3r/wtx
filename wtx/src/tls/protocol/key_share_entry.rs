use crate::{
  codec::{Decode, Encode},
  misc::Lease,
  tls::{
    NamedGroup, TlsError, de::De, misc::u16_chunk, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KeyShareEntry<B> {
  pub(crate) group: NamedGroup,
  pub(crate) opaque: B,
}

impl<B> KeyShareEntry<B> {
  pub(crate) fn new(group: NamedGroup, opaque: B) -> Self {
    Self { group, opaque }
  }
}

impl<'de, B> Decode<'de, De> for KeyShareEntry<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let group = NamedGroup::decode(dw)?;
    let opaque = if dw.is_hello_retry_request() {
      &[][..]
    } else {
      u16_chunk(dw, TlsError::InvalidKeyShareEntry, |el| Ok(el.bytes()))?
    };
    Ok(Self { group, opaque: opaque.try_into().map_err(Into::into)? })
  }
}

impl<B> Encode<De> for KeyShareEntry<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    if ew.is_hello_retry_request() {
      ew.buffer().extend_from_copyable_slice(&u16::from(self.group).to_be_bytes())?;
    } else {
      let opaque = self.opaque.lease();
      let _ = ew.buffer().extend_from_copyable_slices([
        &u16::from(self.group).to_be_bytes(),
        &u16::try_from(opaque.len())?.to_be_bytes(),
        opaque,
      ])?;
    }
    Ok(())
  }
}
