use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorU8,
  misc::counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write_iter},
  tls::{
    TlsError, de::De, misc::u16_list, protocol::server_name::ServerName,
    tls_decode_wrapper::TlsDecodeWrapper, tls_encode_wrapper::TlsEncodeWrapper,
  },
};

/// A mechanism for a client to tell a server the name of the server it is contacting.
///
/// <https://www.rfc-editor.org/info/rfc6066/#section-3>
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ServerNameList {
  /// See [`ServerName`].
  pub server_name_list: ArrayVectorU8<ServerName, 1>,
}

impl<'de> Decode<'de, De> for ServerNameList {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    if dw.bytes().is_empty() {
      return Ok(Self { server_name_list: ArrayVectorU8::new() });
    }
    let mut server_name_list = ArrayVectorU8::new();
    u16_list(&mut server_name_list, dw, TlsError::InvalidServerNameList)?;
    Ok(Self { server_name_list })
  }
}

impl Encode<De> for ServerNameList {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    if self.server_name_list.is_empty() {
      return crate::Result::Ok(());
    }
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      &self.server_name_list,
      None,
      ew,
      |elem, local_ew| {
        elem.encode(local_ew)?;
        crate::Result::Ok(())
      },
    )
  }
}
