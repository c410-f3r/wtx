use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  misc::{
    counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u8_write_iter},
  },
  tls::{TlsError, de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper, misc::u8_list},
};

create_enum! {
  #[derive(Clone, Copy, Debug, PartialEq)]
  pub enum PskKeyExchangeMode<u8> {
    PskDheKe = (1),
  }
}

impl<'de> Decode<'de, De> for PskKeyExchangeMode {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    Ok(Self::try_from(<u8 as Decode<'de, De>>::decode(dw)?)?)
  }
}

impl Encode<De> for PskKeyExchangeMode {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_byte(u8::from(*self))
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
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let mut modes = ArrayVectorU8::new();
    u8_list(&mut modes, dw, TlsError::InvalidPskKeyExchangeModes)?;
    Ok(Self { modes })
  }
}

impl Encode<De> for PskKeyExchangeModes {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    u8_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      &self.modes,
      None,
      ew,
      |el, local_ew| el.encode(local_ew),
    )
  }
}
