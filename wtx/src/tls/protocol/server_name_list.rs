use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorU8,
  misc::{
    Lease,
    counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write_iter},
  },
  tls::{
    TlsError, de::De, misc::u16_list, protocol::server_name::ServerName,
    tls_decode_wrapper::TlsDecodeWrapper, tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ServerNameList<B> {
  pub(crate) names: ArrayVectorU8<ServerName<B>, 1>,
}

impl<'de, B> Decode<'de, De> for ServerNameList<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let mut names = ArrayVectorU8::new();
    u16_list(&mut names, dw, TlsError::InvalidServerNameList)?;
    Ok(Self { names })
  }
}

impl<B> Encode<De> for ServerNameList<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
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
