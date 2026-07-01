use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorU8,
  misc::counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u8_write_iter},
  tls::{
    TlsError, de::De, misc::u8_list, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

create_enum! {
  #[derive(Clone, Copy, Debug, PartialEq)]
  pub(crate) enum PskKeyExchangeMode<u8> {
    PskDheKe = (1),
  }
}

impl<'de> Decode<'de, De> for PskKeyExchangeMode {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    Self::try_from(<u8 as Decode<'de, De>>::decode(dw)?)
  }
}

impl Encode<De> for PskKeyExchangeMode {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().push(u8::from(*self))
  }
}

#[derive(Debug, PartialEq)]
pub(crate) struct PskKeyExchangeModes {
  pub(crate) modes: ArrayVectorU8<PskKeyExchangeMode, 1>,
}

impl PskKeyExchangeModes {
  pub(crate) fn new(modes: ArrayVectorU8<PskKeyExchangeMode, 1>) -> Self {
    Self { modes }
  }
}

impl<'de> Decode<'de, De> for PskKeyExchangeModes {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let mut modes = ArrayVectorU8::new();
    u8_list(&mut modes, dw, TlsError::InvalidPskKeyExchangeModes)?;
    Ok(Self { modes })
  }
}

#[expect(clippy::redundant_closure_for_method_calls, reason = "false-positive")]
impl Encode<De> for PskKeyExchangeModes {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    u8_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      &self.modes,
      None,
      ew,
      |el, local_ew| el.encode(local_ew),
    )
  }
}
