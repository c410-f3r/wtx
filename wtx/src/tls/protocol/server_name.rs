use crate::{
  codec::{Decode, Encode},
  collections::ArrayStringU8,
  misc::{
    Lease as _,
    counter_writer::{CounterWriterBytesTy, u16_write},
  },
  tls::{
    TlsError, de::De, protocol::name_type::NameType, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

/// Server name
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerName {
  pub(crate) name_type: NameType,
  pub(crate) name: ArrayStringU8<14>,
}

impl ServerName {
  /// From arbitrary name
  #[inline]
  pub const fn from_name(name: ArrayStringU8<14>) -> Self {
    Self { name_type: NameType::HostName, name }
  }

  /// Name
  #[inline]
  pub const fn name(&self) -> &ArrayStringU8<14> {
    &self.name
  }
}

impl<'de> Decode<'de, De> for ServerName {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let name_type = NameType::decode(dw)?;
    let len: u16 = Decode::<'_, De>::decode(dw)?;
    let Some((name, rest)) = dw.bytes().split_at_checked(len.into()) else {
      return Err(TlsError::InvalidServerName.into());
    };
    *dw.bytes_mut() = rest;
    Ok(Self { name_type, name: name.try_into()? })
  }
}

impl Encode<De> for ServerName {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    self.name_type.encode(ew)?;
    u16_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      local_ew.buffer().extend_from_copyable_slice(self.name.lease())?;
      Ok(())
    })
  }
}
