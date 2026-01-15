use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  misc::{
    SuffixWriterMut,
    counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write_iter},
  },
  tls::{TlsError, de::De, misc::u16_list, protocol::server_name::ServerName},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerNameList<'any> {
  pub names: ArrayVectorU8<ServerName<'any>, 1>,
}

impl<'de> Decode<'de, De> for ServerNameList<'de> {
  #[inline]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let mut names = ArrayVectorU8::new();
    u16_list(&mut names, dw, TlsError::InvalidServerNameList)?;
    Ok(Self { names })
  }
}

impl Encode<De> for ServerNameList<'_> {
  #[inline]
  fn encode(&self, sw: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      &self.names,
      None,
      sw,
      |elem, local_sw| {
        elem.encode(local_sw)?;
        crate::Result::Ok(())
      },
    )
  }
}
