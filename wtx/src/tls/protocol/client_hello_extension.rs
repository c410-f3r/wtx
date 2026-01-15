use crate::{
  de::{Decode, Encode},
  misc::{
    SuffixWriterMut,
    counter_writer::{CounterWriterBytesTy, u16_write},
  },
  tls::{
    TlsError, de::De, misc::u16_chunk, protocol::client_hello_extension_ty::ClientHelloExtensionTy,
  },
};

pub(crate) struct ClientHelloExtension<T> {
  data: T,
  extension_ty: ClientHelloExtensionTy,
}

impl<T> ClientHelloExtension<T> {
  pub(crate) fn new(extension_ty: ClientHelloExtensionTy, data: T) -> Self {
    Self { data, extension_ty }
  }

  pub(crate) fn into_data(self) -> T {
    self.data
  }
}

impl<'de, T> Decode<'de, De> for ClientHelloExtension<T>
where
  T: Decode<'de, De>,
{
  #[inline]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let extension_ty = ClientHelloExtensionTy::decode(dw)?;
    let data = u16_chunk(dw, TlsError::InvalidClientHelloExtension, |chunk| T::decode(chunk))?;
    Ok(Self { data, extension_ty })
  }
}

impl<T> Encode<De> for ClientHelloExtension<T>
where
  T: Encode<De>,
{
  #[inline]
  fn encode(&self, sw: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    self.extension_ty.encode(sw)?;
    u16_write(CounterWriterBytesTy::IgnoresLen, None, sw, |local_ew| {
      self.data.encode(local_ew)?;
      Ok(())
    })
  }
}
