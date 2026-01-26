use crate::{
  de::{Decode, Encode},
  misc::{
    SuffixWriterMut,
    counter_writer::{CounterWriterBytesTy, u16_write},
  },
  tls::{TlsError, de::De, misc::u16_chunk, protocol::extension_ty::ExtensionTy},
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
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let extension_ty = ExtensionTy::decode(dw)?;
    let data = u16_chunk(dw, TlsError::InvalidExtension, |chunk| T::decode(chunk))?;
    Ok(Self { data, extension_ty })
  }
}

impl<T> Encode<De> for Extension<T>
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
