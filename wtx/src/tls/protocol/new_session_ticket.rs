use crate::{
  de::{Decode, Encode},
  tls::{
    TlsError, de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper,
    misc::u16_chunk, protocol::extension_ty::ExtensionTy,
  },
};

#[derive(Debug)]
pub struct NewSessionTicket;

impl<'de> Decode<'de, De> for NewSessionTicket {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    u16_chunk(dw, TlsError::InvalidNewSessionTicket, |local_dw| {
      while !local_dw.bytes().is_empty() {
        let extension_ty = {
          let tmp_bytes = &mut *local_dw;
          ExtensionTy::decode(tmp_bytes)?
        };
        match extension_ty {
          ExtensionTy::EarlyData => {
            return Err(TlsError::UnsupportedExtension.into());
          }
          _ => {
            return Err(TlsError::MismatchedExtension.into());
          }
        }
      }
      Ok(())
    })?;
    Ok(Self)
  }
}

impl Encode<De> for NewSessionTicket {
  #[inline]
  fn encode(&self, _: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    Ok(())
  }
}
