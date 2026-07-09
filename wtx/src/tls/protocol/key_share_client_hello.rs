use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorU8,
  misc::counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write_iter},
  tls::{
    MAX_KEY_SHARES_LEN, NamedGroup, TlsError, de::De, misc::u16_chunk,
    protocol::key_share_entry::KeyShareEntry, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Debug, PartialEq)]
pub(crate) struct KeyShareClientHello<'any> {
  pub(crate) client_shares: ArrayVectorU8<KeyShareEntry<&'any [u8]>, MAX_KEY_SHARES_LEN>,
}

impl<'de> Decode<'de, De> for KeyShareClientHello<'de> {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let mut client_shares = ArrayVectorU8::new();
    *dw.bytes_mut() = u16_chunk(dw, TlsError::InvalidCipherSuite, |el| Ok(el.bytes()))?;
    while let [b0, b1, rest @ ..] = dw.bytes() {
      let group_rslt = NamedGroup::try_from(u16::from_be_bytes([*b0, *b1]));
      *dw.bytes_mut() = rest;
      let opaque = u16_chunk(dw, TlsError::InvalidKeyShareEntry, |el| Ok(el.bytes()))?;
      let Ok(group) = group_rslt else {
        continue;
      };
      client_shares.push(KeyShareEntry::new(group, opaque))?;
    }
    Ok(Self { client_shares })
  }
}

impl Encode<De> for KeyShareClientHello<'_> {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      &self.client_shares,
      None,
      ew,
      |el, local_ew| {
        el.encode(local_ew)?;
        crate::Result::Ok(())
      },
    )?;
    Ok(())
  }
}
