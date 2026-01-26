use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  misc::{
    counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write_iter},
  },
  tls::{
    TlsError, de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper, misc::u16_list, protocol::server_name::ServerName
  },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerNameList<'any> {
  pub names: ArrayVectorU8<ServerName<'any>, 1>,
}

impl<'de> Decode<'de, De> for ServerNameList<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let mut names = ArrayVectorU8::new();
    u16_list(&mut names, dw, TlsError::InvalidServerNameList)?;
    Ok(Self { names })
  }
}

impl Encode<De> for ServerNameList<'_> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      &self.names,
      None,
      ew,
      |elem, local_ew| {
        elem.encode(local_ew)?;
        crate::Result::Ok(())
      },
    )
  }
}
