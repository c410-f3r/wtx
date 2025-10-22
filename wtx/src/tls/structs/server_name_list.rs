use crate::{
  de::{Decode, Encode},
  misc::{
    SuffixWriterMut,
    counter_writer::{CounterWriter, U16Counter},
  },
  tls::{TlsError, de::De, misc::u16_chunk, structs::server_name::ServerName},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerNameList<'any> {
  pub names: ServerName<'any>,
}

impl<'de> Decode<'de, De> for ServerNameList<'de> {
  #[inline]
  fn decode(aux: &mut (), dw: &mut &'de [u8]) -> crate::Result<Self> {
    Ok(Self {
      names: u16_chunk(dw, TlsError::InvalidClientHello, |chunk| ServerName::decode(aux, chunk))?,
    })
  }
}

impl Encode<De> for ServerNameList<'_> {
  #[inline]
  fn encode(&self, aux: &mut (), sw: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    U16Counter::default().write(sw, false, None, |local_sw| {
      self.names.encode(aux, local_sw)?;
      Ok(())
    })
  }
}
