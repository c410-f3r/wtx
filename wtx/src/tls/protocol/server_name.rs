use crate::{
  codec::{Decode, Encode},
  misc::{
    Lease,
    counter_writer::{CounterWriterBytesTy, u16_write},
  },
  tls::{
    TlsError, de::De, protocol::name_type::NameType, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ServerName<B> {
  pub(crate) name_type: NameType,
  pub(crate) name: B,
}

impl<'de, B> Decode<'de, De> for ServerName<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let name_type = NameType::decode(dw)?;
    let len: u16 = Decode::<'_, De>::decode(dw)?;
    let Some((name, rest)) = dw.bytes().split_at_checked(len.into()) else {
      return Err(TlsError::InvalidServerName.into());
    };
    *dw.bytes_mut() = rest;
    Ok(Self { name_type, name: name.try_into().map_err(Into::into)? })
  }
}

impl<B> Encode<De> for ServerName<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    self.name_type.encode(ew)?;
    u16_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      local_ew.buffer().extend_from_copyable_slice(self.name.lease())?;
      Ok(())
    })
  }
}
