use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  misc::{
    SuffixWriterMut,
    counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u8_write_iter},
  },
  tls::{TlsError, de::De, misc::u8_list},
};

create_enum! {
  #[derive(Clone, Copy, Debug, PartialEq)]
  pub enum PskKeyExchangeMode<u8> {
    PskDheKe = (1),
  }
}

impl<'de> Decode<'de, De> for PskKeyExchangeMode {
  #[inline]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    Ok(Self::try_from(<u8 as Decode<'de, De>>::decode(dw)?)?)
  }
}

impl Encode<De> for PskKeyExchangeMode {
  #[inline]
  fn encode(&self, ew: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    ew.extend_from_byte(u8::from(*self))
  }
}

#[derive(Debug, PartialEq)]
pub struct PskKeyExchangeModes {
  pub modes: ArrayVectorU8<PskKeyExchangeMode, 1>,
}

impl PskKeyExchangeModes {
  pub fn new(modes: ArrayVectorU8<PskKeyExchangeMode, 1>) -> Self {
    Self { modes }
  }
}

impl<'de> Decode<'de, De> for PskKeyExchangeModes {
  #[inline]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let mut modes = ArrayVectorU8::new();
    u8_list(&mut modes, dw, TlsError::InvalidPskKeyExchangeModes)?;
    Ok(Self { modes })
  }
}

impl Encode<De> for PskKeyExchangeModes {
  #[inline]
  fn encode(&self, ew: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    u8_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      &self.modes,
      None,
      ew,
      |el, local_ew| el.encode(local_ew),
    )
  }
}
