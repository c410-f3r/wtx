use crate::{
  misc::{
    Decode, Encode, SuffixWriterMut,
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
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    Ok(Self {
      names: u16_chunk(dw, TlsError::InvalidClientHello, |chunk| ServerName::decode(chunk))?,
    })
  }
}

impl Encode<De> for ServerNameList<'_> {
  #[inline]
  fn encode(&self, ew: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    U16Counter::write(ew, false, None, |local_ew| {
      self.names.encode(local_ew)?;
      Ok(())
    })
  }
}
