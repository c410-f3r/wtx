use crate::{
  de::{Decode, Encode},
  misc::{
    SuffixWriterMut,
    counter_writer::{CounterWriter, U16Counter},
    from_utf8_basic,
  },
  tls::{TlsError, de::De, structs::name_type::NameType},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerName<'any> {
  pub name_type: NameType,
  pub name: &'any str,
}

impl<'de> Decode<'de, De> for ServerName<'de> {
  #[inline]
  fn decode(aux: &mut (), dw: &mut &'de [u8]) -> crate::Result<Self> {
    let name_type = NameType::decode(aux, dw)?;
    let len: u16 = Decode::<'_, De>::decode(aux, dw)?;
    let Some((name, rest)) = dw.split_at_checked(len.into()) else {
      return Err(TlsError::InvalidServerName.into());
    };
    *dw = rest;
    Ok(Self { name_type, name: from_utf8_basic(name)? })
  }
}

impl Encode<De> for ServerName<'_> {
  #[inline]
  fn encode(&self, aux: &mut (), ew: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    self.name_type.encode(aux, ew)?;
    U16Counter::default().write(ew, false, None, |local_ew| {
      local_ew.extend_from_slice(self.name.as_bytes())?;
      Ok(())
    })
  }
}
