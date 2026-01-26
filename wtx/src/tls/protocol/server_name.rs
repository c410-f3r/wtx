use crate::{
  de::{Decode, Encode},
  misc::{
    counter_writer::{CounterWriterBytesTy, u16_write},
    from_utf8_basic,
  },
  tls::{
    TlsError, de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper,
    protocol::name_type::NameType,
  },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerName<'any> {
  pub name_type: NameType,
  pub name: &'any str,
}

impl<'de> Decode<'de, De> for ServerName<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let name_type = NameType::decode(dw)?;
    let len: u16 = Decode::<'_, De>::decode(dw)?;
    let Some((name, rest)) = dw.bytes().split_at_checked(len.into()) else {
      return Err(TlsError::InvalidServerName.into());
    };
    *dw.bytes_mut() = rest;
    Ok(Self { name_type, name: from_utf8_basic(name)? })
  }
}

impl Encode<De> for ServerName<'_> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    self.name_type.encode(ew)?;
    u16_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      local_ew.buffer().extend_from_slice(self.name.as_bytes())?;
      Ok(())
    })
  }
}
