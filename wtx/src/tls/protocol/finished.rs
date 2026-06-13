// https://datatracker.ietf.org/doc/html/rfc8446#section-4.4.4

use crate::{
  codec::{Decode, Encode},
  tls::{de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper},
};

#[derive(Debug)]
pub struct Finished<'any> {
  verify_data: &'any [u8],
}

impl<'any> Finished<'any> {
  pub fn verify_data(&self) -> &'any [u8] {
    self.verify_data
  }
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
