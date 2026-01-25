use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  misc::{
    SuffixWriterMut,
    counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write_iter},
  },
  tls::{
    MAX_KEY_SHARES_LEN, TlsError, de::De, misc::u16_list, protocol::key_share_entry::KeyShareEntry,
  },
};

#[derive(Debug, PartialEq)]
pub struct KeyShareClientHello<'any> {
  pub client_shares: ArrayVectorU8<KeyShareEntry<'any>, MAX_KEY_SHARES_LEN>,
}

impl<'de> Decode<'de, De> for KeyShareClientHello<'de> {
  #[inline]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let mut client_shares = ArrayVectorU8::new();
    u16_list(&mut client_shares, dw, TlsError::InvalidKeyShareClientHello)?;
    Ok(Self { client_shares })
  }
}

impl<'any> Encode<De> for KeyShareClientHello<'any> {
  #[inline]
  fn encode(&self, ew: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
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
