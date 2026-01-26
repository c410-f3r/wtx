use crate::{
  de::{Decode, Encode},
  misc::counter_writer::{CounterWriterBytesTy, u16_write},
  tls::{
    TlsError, de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper,
    misc::u16_chunk, protocol::extension_ty::ExtensionTy,
  },
};

pub(crate) struct Extension<T> {
  data: T,
  extension_ty: ExtensionTy,
}

impl<T> Extension<T> {
  pub(crate) fn new(extension_ty: ExtensionTy, data: T) -> Self {
    Self { data, extension_ty }
  }

  pub(crate) fn into_data(self) -> T {
    self.data
  }
}

impl<'de, T> Decode<'de, De> for Extension<T>
where
  T: Decode<'de, De>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let extension_ty = ExtensionTy::decode(dw)?;
    let data = u16_chunk(dw, TlsError::InvalidExtension, |local_dw| T::decode(local_dw))?;
    Ok(Self { data, extension_ty })
  }
}

impl<T> Encode<De> for Extension<T>
where
  T: Encode<De>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    self.extension_ty.encode(ew)?;
    u16_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      self.data.encode(local_ew)?;
      Ok(())
    })
  }
}
