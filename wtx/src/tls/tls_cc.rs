use crate::{
  codec::{CodecController, Decode, Encode},
  tls::{
    TlsError,
    tls_cc_wrappers::{TlsDecodeWrapper, TlsEncodeWrapper},
  },
};

pub(crate) struct TlsCc;

impl CodecController for TlsCc {
  type DecodeWrapper<'inner, 'outer, 'misc>
    = TlsDecodeWrapper<'inner>
  where
    'inner: 'outer;
  type Error = crate::Error;
  type EncodeWrapper<'inner, 'outer, 'misc>
    = TlsEncodeWrapper<'inner>
  where
    'inner: 'outer;
}

impl<'de> Decode<'de, TlsCc> for &'de [u8] {
  #[inline]
  #[track_caller]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    Ok(dw.bytes())
  }
}

impl<'de> Decode<'de, TlsCc> for u8 {
  #[inline]
  #[track_caller]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let [b0, rest @ ..] = dw.bytes() else {
      return Err(TlsError::InvalidU8Prefix.into());
    };
    *dw.bytes_mut() = rest;
    Ok(*b0)
  }
}

impl<'de> Decode<'de, TlsCc> for u16 {
  #[inline]
  #[track_caller]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let [b0, b1, rest @ ..] = dw.bytes() else {
      return Err(TlsError::InvalidU16Prefix.into());
    };
    *dw.bytes_mut() = rest;
    Ok(u16::from_be_bytes([*b0, *b1]))
  }
}

impl<'de> Decode<'de, TlsCc> for u32 {
  #[inline]
  #[track_caller]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let [b0, b1, b2, b3, rest @ ..] = dw.bytes() else {
      return Err(TlsError::InvalidU16Prefix.into());
    };
    *dw.bytes_mut() = rest;
    Ok(u32::from_be_bytes([*b0, *b1, *b2, *b3]))
  }
}

impl<'de, const N: usize> Decode<'de, TlsCc> for [u8; N] {
  #[inline]
  #[track_caller]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let Some((lhs, rhs)) = dw.bytes().split_at_checked(N) else {
      return Err(TlsError::InvalidArray.into());
    };
    *dw.bytes_mut() = rhs;
    Ok(lhs.try_into()?)
  }
}

impl Encode<TlsCc> for [u8] {
  #[inline]
  #[track_caller]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_copyable_slice(self)?;
    Ok(())
  }
}

impl Encode<TlsCc> for u16 {
  #[inline]
  #[track_caller]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_copyable_slice(&self.to_be_bytes())?;
    Ok(())
  }
}

impl Encode<TlsCc> for u32 {
  #[inline]
  #[track_caller]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_copyable_slice(&self.to_be_bytes())?;
    Ok(())
  }
}
