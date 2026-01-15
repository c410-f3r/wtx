// https://datatracker.ietf.org/doc/html/rfc8446#section-4.2.11

use crate::{
  de::{Decode, Encode},
  misc::{
    SuffixWriterMut,
    counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write, u16_write_iter},
  },
  tls::de::De,
};

#[derive(Debug, PartialEq)]
pub struct PreSharedKeyClientHello<'any> {
  pub identities: &'any [&'any [u8]],
  pub binder: &'any [&'any [u8]],
}

impl Encode<De> for PreSharedKeyClientHello<'_> {
  #[inline]
  fn encode(&self, sw: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      self.identities,
      None,
      sw,
      |elem, local_sw| {
        u16_write(CounterWriterBytesTy::IgnoresLen, None, local_sw, |local_local_sw| {
          local_local_sw.extend_from_slice(elem)?;
          crate::Result::Ok(())
        })
      },
    )?;
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      self.identities,
      None,
      sw,
      |elem, local_sw| {
        u16_write(CounterWriterBytesTy::IgnoresLen, None, local_sw, |local_local_sw| {
          local_local_sw.extend_from_slice(elem)?;
          crate::Result::Ok(())
        })
      },
    )?;
    Ok(())
  }
}

#[derive(Debug, PartialEq)]
pub(crate) struct PreSharedKeyServerHello {
  pub(crate) selected_identity: u16,
}

impl<'de> Decode<'de, De> for PreSharedKeyServerHello {
  #[inline]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let selected_identity: u16 = Decode::<'_, De>::decode(dw)?;
    Ok(Self { selected_identity })
  }
}
