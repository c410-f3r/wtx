// https://datatracker.ietf.org/doc/html/rfc8446#section-4.4.4

use crate::{
  de::{Decode, Encode},
  tls::{de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper},
};

#[derive(Debug)]
pub struct Finished<'any> {
  verify_data: &'any [u8],
}

impl<'de> Decode<'de, De> for Finished<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    todo!()
  }
}

impl<'any> Encode<De> for Finished<'any> {
  #[inline]
  fn encode(&self, _: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    Ok(())
  }
}
